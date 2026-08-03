import { afterEach, describe, expect, test } from "vitest";

import { extractAppearance } from "./rasters";
import {
  clearRasterStore,
  getRasterInfo,
  isRasterHandle,
  RASTER_REF,
  releaseOwner,
  retainedRasterBytes,
} from "./rasterStore";

const OWNER = "https://example.test/node.out.jsonl.zst";

/** A PNG header followed by filler, as the engine writes it: an array of ints. */
function pngBytes(length: number, fill = 7): number[] {
  const bytes = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
  while (bytes.length < length) bytes.push(fill);
  return bytes.slice(0, length);
}

function inMemoryTexture(bytes: number[]) {
  return {
    raster: { InMemory: { mime_type: "image/png", bytes } },
    sampler: {
      wrap_s: "Repeat",
      wrap_t: "Repeat",
      mag_filter: "Linear",
      min_filter: "LinearMipmap",
    },
    uv_channel: 0,
  };
}

function meshWithTexture(bytes: number[]) {
  return {
    Euclidean3D: {
      PolygonMesh: {
        frame: { Crs: 4979 },
        faces: [{ exterior: [0, 1, 2] }],
        appearance: {
          materials: [{ Pbr: { base_color_map: inMemoryTexture(bytes) } }],
          themes: [],
          default_theme: "default",
        },
      },
    },
  };
}

afterEach(() => clearRasterStore());

describe("extractAppearance lifts embedded images out of the geometry", () => {
  test("replaces embedded bytes with a handle and stores the image", () => {
    const geometry = meshWithTexture(pngBytes(64));

    const handles = extractAppearance(geometry, OWNER).textures;

    expect(handles).toHaveLength(1);
    const stored =
      geometry.Euclidean3D.PolygonMesh.appearance.materials[0].Pbr
        .base_color_map.raster.InMemory;
    expect(isRasterHandle(stored)).toBe(true);
    expect(stored).toMatchObject({ mime_type: "image/png", byteLength: 64 });
    expect(Array.isArray((stored as { bytes?: unknown }).bytes)).toBe(false);

    const info = getRasterInfo(handles[0][RASTER_REF]);
    expect(info).toMatchObject({
      mime: "image/png",
      byteLength: 64,
      retained: true,
    });
  });

  test("collapses the same image shared across features onto one entry", () => {
    const first = meshWithTexture(pngBytes(512));
    const second = meshWithTexture(pngBytes(512));

    const a = extractAppearance(first, OWNER).textures;
    const b = extractAppearance(second, OWNER).textures;

    expect(a[0][RASTER_REF]).toBe(b[0][RASTER_REF]);
    expect(retainedRasterBytes()).toBe(512);
  });

  test("keeps distinct images apart", () => {
    const a = extractAppearance(
      meshWithTexture(pngBytes(512, 1)),
      OWNER,
    ).textures;
    const b = extractAppearance(
      meshWithTexture(pngBytes(512, 2)),
      OWNER,
    ).textures;

    expect(a[0][RASTER_REF]).not.toBe(b[0][RASTER_REF]);
    expect(retainedRasterBytes()).toBe(1024);
  });

  test("leaves a Uri raster untouched", () => {
    const geometry = {
      Euclidean3D: {
        Polygon: {
          frame: { Crs: 4979 },
          exterior: [],
          appearance: {
            materials: [
              { Phong: { diffuse_map: { raster: { Uri: "file:///t.jpg" } } } },
            ],
            themes: [],
            default_theme: "default",
          },
        },
      },
    };

    expect(extractAppearance(geometry, OWNER).textures).toHaveLength(0);
    expect(
      geometry.Euclidean3D.Polygon.appearance.materials[0].Phong.diffuse_map
        .raster,
    ).toEqual({ Uri: "file:///t.jpg" });
  });

  test("reaches textures on a solid's shells", () => {
    const geometry = {
      Euclidean3D: {
        Solid: {
          frame: { Crs: 4979 },
          exterior: {
            PolygonMesh: {
              faces: [],
              appearance: {
                materials: [
                  { Pbr: { emissive_map: inMemoryTexture(pngBytes(32)) } },
                ],
                themes: [],
                default_theme: "default",
              },
            },
          },
        },
      },
    };

    expect(extractAppearance(geometry, OWNER).textures).toHaveLength(1);
  });

  test("reaches textures through CSG tuple operands", () => {
    const shell = (bytes: number[]) => ({
      PolygonMesh: {
        faces: [],
        appearance: {
          materials: [{ Pbr: { base_color_map: inMemoryTexture(bytes) } }],
          themes: [],
          default_theme: "default",
        },
      },
    });
    const geometry = {
      Euclidean3D: {
        Csg: {
          Difference: [
            {
              Solid: { frame: { Crs: 4979 }, exterior: shell(pngBytes(16, 3)) },
            },
            {
              Solid: { frame: { Crs: 4979 }, exterior: shell(pngBytes(16, 4)) },
            },
          ],
        },
      },
    };

    expect(extractAppearance(geometry, OWNER).textures).toHaveLength(2);
  });

  test("reaches textures nested in a geometry collection", () => {
    const geometry = {
      GeometryCollection: {
        members: ["None", meshWithTexture(pngBytes(24))],
      },
    };

    expect(extractAppearance(geometry, OWNER).textures).toHaveLength(1);
  });

  test("ignores geometry with no appearance at all", () => {
    const geometry = {
      Euclidean2D: {
        Point: { frame: { Crs: 4326 }, position: [139.7, 35.6] },
      },
    };

    expect(extractAppearance(geometry, OWNER).textures).toHaveLength(0);
    expect(geometry.Euclidean2D.Point.position).toEqual([139.7, 35.6]);
  });

  test("survives malformed and legacy-shaped geometry", () => {
    expect(extractAppearance("None", OWNER).textures).toHaveLength(0);
    expect(extractAppearance(null, OWNER).textures).toHaveLength(0);
    expect(
      extractAppearance({ epsg: 4326, value: { flowGeometry2D: {} } }, OWNER)
        .textures,
    ).toHaveLength(0);
    expect(
      extractAppearance({ Euclidean3D: { PolygonMesh: null } }, OWNER).textures,
    ).toHaveLength(0);
  });
});

describe("extractAppearance summarizes materials", () => {
  test("labels each material and its texture slots from the schema", () => {
    const geometry = {
      Euclidean3D: {
        Polygon: {
          frame: { Crs: 4979 },
          exterior: [],
          appearance: {
            materials: [
              {
                Pbr: {
                  base_color_map: inMemoryTexture(pngBytes(64)),
                  normal_map: { raster: { Uri: "file:///normals.png" } },
                },
              },
              { Phong: { diffuse_map: inMemoryTexture(pngBytes(48)) } },
            ],
            themes: [],
            default_theme: "default",
          },
        },
      },
    };

    const { materials, textures } = extractAppearance(geometry, OWNER);

    expect(materials).toHaveLength(2);
    expect(materials[0]).toMatchObject({ kind: "Pbr", label: "PBR material" });
    expect(materials[0].textures.map((texture) => texture.slot)).toEqual([
      "base_color_map",
      "normal_map",
    ]);
    expect(materials[0].textures[0].image).toBeDefined();
    expect(materials[0].textures[1]).toMatchObject({
      uri: "file:///normals.png",
    });
    expect(materials[0].textures[1].image).toBeUndefined();
    expect(materials[1]).toMatchObject({
      kind: "Phong",
      label: "Phong material",
    });
    expect(textures).toHaveLength(2);
  });

  test("reports a material with no maps rather than dropping it", () => {
    const geometry = {
      Euclidean3D: {
        Polygon: {
          frame: { Crs: 4979 },
          exterior: [],
          appearance: {
            materials: [{ Phong: { diffuse: [1, 1, 1] } }],
            themes: [],
            default_theme: "default",
          },
        },
      },
    };

    const { materials } = extractAppearance(geometry, OWNER);

    expect(materials).toHaveLength(1);
    expect(materials[0].textures).toEqual([]);
  });

  test("finds nothing on geometry that carries no appearance", () => {
    const geometry = {
      Euclidean2D: { Point: { frame: { Crs: 4326 }, position: [1, 2] } },
    };

    expect(extractAppearance(geometry, OWNER)).toEqual({
      materials: [],
      textures: [],
    });
  });
});

describe("releaseOwner", () => {
  test("frees an image once no cached file references it", () => {
    const other = "https://example.test/other.jsonl";
    extractAppearance(meshWithTexture(pngBytes(256)), OWNER);
    extractAppearance(meshWithTexture(pngBytes(256)), other);

    releaseOwner(OWNER);
    expect(retainedRasterBytes()).toBe(256);

    releaseOwner(other);
    expect(retainedRasterBytes()).toBe(0);
  });
});
