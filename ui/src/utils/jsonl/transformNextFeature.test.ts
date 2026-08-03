import { afterEach, describe, expect, test } from "vitest";

import { clearRasterStore } from "@flow/lib/intermediateData";

import { intermediateDataTransform } from "./transformIntermediateData";
import { transformNextFeature } from "./transformNextFeature";

const OWNER = "https://example.test/node.out.jsonl.zst";

const feature = (
  geometry: unknown,
  attributes: Record<string, unknown> = {},
) => ({
  id: "0195f3a0-0000-7000-8000-000000000000",
  attributes,
  geometry,
});

afterEach(() => clearRasterStore());

describe("2D geometry becomes GeoJSON", () => {
  test("point", () => {
    const result = transformNextFeature(
      feature({
        Euclidean2D: {
          Point: { frame: { Crs: 4326 }, position: [139.7, 35.6] },
        },
      }),
      { owner: OWNER },
    );

    expect(result.geometry).toEqual({
      type: "Point",
      coordinates: [139.7, 35.6],
      frame: "EPSG:4326",
    });
  });

  test("polyline lifts its single elevation onto every coordinate", () => {
    const result = transformNextFeature(
      feature({
        Euclidean2D: {
          LineString: {
            frame: { Crs: 4326 },
            coords: [
              [0, 0],
              [1, 1],
            ],
            z: 12.5,
          },
        },
      }),
      { owner: OWNER },
    );

    expect(result.geometry).toMatchObject({
      type: "LineString",
      coordinates: [
        [0, 0, 12.5],
        [1, 1, 12.5],
      ],
    });
  });

  test("polygon closes an open ring and keeps its holes", () => {
    const result = transformNextFeature(
      feature({
        Euclidean2D: {
          Polygon: {
            frame: "Euclidean",
            exterior: [
              [0, 0],
              [2, 0],
              [2, 2],
            ],
            interiors: [
              [
                [0.5, 0.5],
                [1, 0.5],
                [1, 1],
                [0.5, 0.5],
              ],
            ],
          },
        },
      }),
      { owner: OWNER },
    );

    const geometry = result.geometry as { coordinates: number[][][] };
    expect(geometry.coordinates[0]).toHaveLength(4);
    expect(geometry.coordinates[0][3]).toEqual([0, 0]);
    // Already closed, so left alone.
    expect(geometry.coordinates[1]).toHaveLength(4);
    expect(result.geometry).toMatchObject({ frame: "Euclidean" });
  });

  test("meshes flatten to a MultiPolygon the 2D map can draw", () => {
    const mesh = transformNextFeature(
      feature({
        Euclidean2D: {
          PolygonMesh: {
            frame: { Crs: 4326 },
            faces: [
              {
                exterior: [
                  [0, 0],
                  [1, 0],
                  [1, 1],
                ],
              },
            ],
          },
        },
      }),
      { owner: OWNER },
    );
    expect(mesh.geometry).toMatchObject({ type: "MultiPolygon" });

    const triangles = transformNextFeature(
      feature({
        Euclidean2D: {
          TriangularMesh: {
            frame: { Crs: 4326 },
            triangles: [
              [
                [0, 0],
                [1, 0],
                [1, 1],
              ],
            ],
          },
        },
      }),
      { owner: OWNER },
    );
    const geometry = triangles.geometry as { coordinates: number[][][][] };
    expect(geometry.coordinates[0][0]).toHaveLength(4);
  });

  test("a mixed collection becomes a GeoJSON GeometryCollection", () => {
    const result = transformNextFeature(
      feature({
        Euclidean2D: {
          Collection: {
            members: [
              { Point: { frame: { Crs: 4326 }, position: [1, 2] } },
              {
                LineString: {
                  frame: { Crs: 4326 },
                  coords: [
                    [0, 0],
                    [1, 1],
                  ],
                },
              },
            ],
          },
        },
      }),
      { owner: OWNER },
    );

    expect(result.geometry).toMatchObject({
      type: "GeometryCollection",
      geometries: [{ type: "Point" }, { type: "LineString" }],
    });
  });

  test("names a tangent frame by its anchor", () => {
    const result = transformNextFeature(
      feature({
        Euclidean2D: {
          Point: {
            frame: {
              Tangent: {
                base: { Crs: 4979 },
                origin: [0, 0, 0],
                u: [1, 0, 0],
                v: [0, 1, 0],
              },
            },
            position: [1, 2],
          },
        },
      }),
      { owner: OWNER },
    );

    expect(result.geometry).toMatchObject({
      frame: "Tangent plane (EPSG:4979)",
    });
  });
});

describe("3D geometry becomes a summary", () => {
  test("reports type, frame and extent without coordinates", () => {
    const result = transformNextFeature(
      feature({
        Euclidean3D: {
          PolygonMesh: {
            frame: { Crs: 4979 },
            faces: [{ exterior: [] }, { exterior: [] }, { exterior: [] }],
          },
        },
      }),
      { owner: OWNER },
    );

    expect(result.geometry).toEqual({
      type: "Polygon mesh (3D)",
      frame: "EPSG:4979",
      summary: "3 faces",
    });
  });

  test("counts point-cloud points across segments and encodings", () => {
    const result = transformNextFeature(
      feature({
        Euclidean3D: {
          PointCloud: {
            frame: { Crs: 4979 },
            segments: [
              {
                positions: {
                  F64: [
                    [0, 0, 0],
                    [1, 1, 1],
                  ],
                },
              },
              {
                positions: {
                  ScaledI32: {
                    scale: [1, 1, 1],
                    offset: [0, 0, 0],
                    values: [[1, 2, 3]],
                  },
                },
              },
            ],
          },
        },
      }),
      { owner: OWNER },
    );

    expect(result.geometry).toMatchObject({
      type: "Point cloud",
      summary: "3 points",
    });
  });

  test("names the boolean operation of a CSG geometry", () => {
    const result = transformNextFeature(
      feature({ Euclidean3D: { Csg: { Difference: [] } } }),
      { owner: OWNER },
    );

    expect(result.geometry).toMatchObject({
      type: "Boolean combination",
      summary: "Difference",
    });
  });

  test("counts the textures lifted out of the geometry", () => {
    const result = transformNextFeature(
      feature({
        Euclidean3D: {
          Polygon: {
            frame: { Crs: 4979 },
            exterior: [],
            appearance: {
              materials: [
                {
                  Pbr: {
                    base_color_map: {
                      raster: {
                        InMemory: {
                          mime_type: "image/png",
                          bytes: [1, 2, 3, 4],
                        },
                      },
                    },
                  },
                },
              ],
              themes: [],
              default_theme: "default",
            },
          },
        },
      }),
      { owner: OWNER },
    );

    expect(result.geometry).toMatchObject({ textures: 1 });
  });
});

describe("features without drawable geometry", () => {
  test("an absent geometry yields attributes only", () => {
    const result = transformNextFeature(
      feature("None", { name: "attributes only" }),
      { owner: OWNER },
    );

    expect(result).toEqual({
      id: "0195f3a0-0000-7000-8000-000000000000",
      type: "Feature",
      properties: { name: "attributes only" },
    });
  });

  test("a geometry collection reports its member count", () => {
    const result = transformNextFeature(
      feature({ GeometryCollection: { members: ["None", "None"] } }),
      { owner: OWNER },
    );

    expect(result.geometry).toMatchObject({ type: "Geometry collection" });
  });
});

describe("intermediateDataTransform dispatch", () => {
  test("routes a legacy feature to the legacy transform", () => {
    const result = intermediateDataTransform({
      id: "1",
      attributes: { a: 1 },
      geometry: {
        epsg: 4326,
        value: { flowGeometry2D: { point: { x: 139.7, y: 35.6 } } },
      },
    });

    expect(result.geometry).toEqual({
      type: "Point",
      coordinates: [139.7, 35.6],
    });
  });

  test("routes a new-format feature to the new transform", () => {
    const result = intermediateDataTransform(
      feature({
        Euclidean2D: { Point: { frame: { Crs: 4326 }, position: [1, 2] } },
      }),
      { owner: OWNER },
    );

    expect(result.geometry).toMatchObject({
      type: "Point",
      frame: "EPSG:4326",
    });
  });

  test("stamps the source line on both formats, for view selection", () => {
    const next = intermediateDataTransform(feature("None"), {
      owner: OWNER,
      rowIndex: 41,
    });
    const legacy = intermediateDataTransform(
      { id: "1", attributes: {}, geometry: { value: "none" } },
      { owner: OWNER, rowIndex: 42 },
    );

    expect(next.rowIndex).toBe(41);
    expect(legacy.rowIndex).toBe(42);
  });
});
