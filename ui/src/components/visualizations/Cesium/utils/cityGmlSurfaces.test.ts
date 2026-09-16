/**
 * The contract between the two CityGML formats and the batched renderer.
 *
 * `cityGmlGeometryToPrimitives` itself constructs Cesium objects, which need a
 * WebGL context, so this covers the part that decides what it draws: whether a
 * feature is routed to it at all, and whether the surfaces it reads come out
 * the same from either transform.
 */
import { describe, expect, test } from "vitest";

import { transformNextFeature } from "@flow/utils/jsonl/transformNextFeature";

import { getFeatureBoundingSphereFromBounds } from "./cesiumFunctions";
import {
  gmlGeometriesOf,
  isCityGmlGeometry,
} from "./cityGmlGeometryToPrimitives";
import { upgradeSurfaces } from "./useLodWorker";

/** The surfaces the renderer will draw, as it reads them. */
function surfaces(geometry: any): any[] {
  return gmlGeometriesOf(geometry)?.flatMap((entry) => entry.polygons) ?? [];
}

/** A CityGML feature as the reader emits it: one member per level of detail. */
function cityGmlFeature(levels: number[]) {
  const solid = (offset: number) => ({
    Euclidean3D: {
      Solid: {
        frame: { Crs: 6697 },
        exterior: {
          PolygonMesh: {
            faces: [
              {
                exterior: [
                  [35.6 + offset, 139.7, 0],
                  [35.6 + offset, 139.8, 0],
                  [35.7 + offset, 139.8, 12],
                ],
              },
            ],
          },
        },
      },
    },
  });

  return {
    id: "0195f3a0-0000-7000-8000-000000000000",
    attributes: { "bldg:measuredHeight": 12 },
    geometry: {
      GeometryCollection: {
        members: levels.map((_, index) => solid(index * 0.1)),
        attrs: levels.map((level) => ({ lod: level })),
      },
    },
  };
}

describe("routing to the batched renderer", () => {
  test("takes CityGML from either transform", () => {
    // Legacy: the engine's record, passed through under its own type.
    expect(
      isCityGmlGeometry({ type: "CityGmlGeometry", gmlGeometries: [] }),
    ).toBe(true);

    // New format: a MultiPolygon like any other, marked by its level of detail.
    const geometry = transformNextFeature(cityGmlFeature([1, 2])).geometry;
    expect(geometry).toMatchObject({ type: "MultiPolygon", lod: 1 });
    expect(isCityGmlGeometry(geometry)).toBe(true);
  });

  test("leaves everything else on the GeoJSON path", () => {
    // A plain mesh is also a MultiPolygon, but it is not CityGML: the entity
    // path gives it styling, and the CityGML one would normalize its heights
    // to the ground and colour it by surface angle.
    const geometry = transformNextFeature({
      id: "x",
      attributes: {},
      geometry: {
        Euclidean3D: {
          PolygonMesh: {
            frame: { Crs: 6697 },
            faces: [
              {
                exterior: [
                  [35.6, 139.7, 0],
                  [35.6, 139.8, 0],
                  [35.7, 139.8, 0],
                ],
              },
            ],
          },
        },
      },
    }).geometry;

    expect(geometry).toMatchObject({ type: "MultiPolygon" });
    expect(isCityGmlGeometry(geometry)).toBe(false);
  });
});

describe("the surfaces the renderer reads", () => {
  test("are the transform's own rings, not a second copy of them", () => {
    // A CityGML file is mostly coordinates. Emitting a parallel structure for
    // the renderer is what exhausted the tab; it reads `coordinates` instead.
    const geometry = transformNextFeature(cityGmlFeature([1])).geometry as any;

    expect(surfaces(geometry)[0].exterior).toBe(geometry.coordinates[0][0]);
  });

  test("carry a usable outer ring, longitude first and closed", () => {
    const geometry = transformNextFeature(cityGmlFeature([1])).geometry as any;
    const [surface] = surfaces(geometry);

    // `coordsToPositions` reads these straight into Cartesian3.fromDegrees,
    // and EPSG:6697 is north-first, so the swap has to have happened already.
    expect(surface.exterior).toEqual([
      [139.7, 35.6, 0],
      [139.8, 35.6, 0],
      [139.8, 35.7, 12],
      [139.7, 35.6, 0],
    ]);
  });

  test("keep the height that decides roof from wall", () => {
    // `getSurfaceTypeColor` and the globalMinZ scan read index 2 of each
    // position; a ring that lost its z reads as flat ground and colours as
    // floor. This surface spans 0 to 12, so it must not read as flat.
    const geometry = transformNextFeature(cityGmlFeature([1])).geometry as any;
    const [surface] = surfaces(geometry);

    const heights = surface.exterior.map((position: number[]) => position[2]);
    expect(Math.max(...heights) - Math.min(...heights)).toBe(12);
  });

  test("one entry per surface, so a building is not one giant polygon", () => {
    const geometry = transformNextFeature({
      id: "x",
      attributes: {},
      geometry: {
        GeometryCollection: {
          members: [
            {
              Euclidean3D: {
                PolygonMesh: {
                  frame: { Crs: 6697 },
                  faces: [
                    {
                      exterior: [
                        [35.6, 139.7, 0],
                        [35.6, 139.8, 0],
                        [35.7, 139.8, 0],
                      ],
                    },
                    {
                      exterior: [
                        [35.6, 139.7, 0],
                        [35.6, 139.8, 0],
                        [35.7, 139.8, 9],
                      ],
                    },
                  ],
                },
              },
            },
          ],
          attrs: [{ lod: 2 }],
        },
      },
    }).geometry as any;

    expect(surfaces(geometry)).toHaveLength(2);
  });
});

describe("the level swapped in when a feature is selected", () => {
  /** A feature as `CityGmlData` holds it, once the transform has run. */
  const drawn = (levels: number[]) => {
    const t = transformNextFeature(cityGmlFeature(levels));
    return { ...t, properties: { _originalId: t.id, ...t.properties } } as any;
  };

  test("takes the finer level the transform kept", () => {
    // Drawn is LOD1's one surface; the upgrade is LOD3's, from `lodDetail`.
    const feature = drawn([1, 2, 3]);
    expect(feature.geometry.lod).toBe(1);

    const surfaces = upgradeSurfaces(feature)?.polygons;
    expect(surfaces).toHaveLength(1);

    // Rings the worker can triangulate: positions, longitude first. The
    // latitude identifies which member this came from — the fixture places
    // each level 0.1 further north, so 35.8 is the third, LOD3.
    const [lon, lat, z] = surfaces?.[0].polygon.exterior[0] as number[];
    expect(lon).toBe(139.7);
    expect(lat).toBeCloseTo(35.8);
    expect(z).toBe(0);
  });

  test("still reads the legacy record, which holds every level itself", () => {
    const legacy = {
      type: "Feature",
      properties: {},
      geometry: {
        type: "CityGmlGeometry",
        gmlGeometries: [
          { lod: 1, pos: 0, polygons: [{ exterior: [{ x: 1, y: 2, z: 0 }] }] },
          {
            lod: 3,
            pos: 1,
            polygons: [
              { exterior: [{ x: 3, y: 4, z: 9 }] },
              { exterior: [{ x: 5, y: 6, z: 9 }] },
            ],
          },
        ],
      },
    } as any;

    const surfaces = upgradeSurfaces(legacy)?.polygons;
    // LOD3's two surfaces, indexed from that entry's `pos`.
    expect(surfaces).toHaveLength(2);
    expect(surfaces?.map((s) => s.globalIndex)).toEqual([1, 2]);
  });

  test("reports nothing to swap in when the drawn level is the finest", () => {
    // `upgradeLod` treats null as "leave it as it is" rather than redrawing
    // the same surfaces on top of themselves.
    expect(upgradeSurfaces(drawn([2]))).toBeNull();
  });
});

describe("the camera reaching a selected feature", () => {
  test("bounds a new-format feature, whose positions are arrays", () => {
    const geometry = transformNextFeature(cityGmlFeature([1])).geometry as any;

    const sphere = getFeatureBoundingSphereFromBounds(
      gmlGeometriesOf(geometry) ?? [],
    );

    expect(sphere).not.toBeNull();
    expect(sphere?.radius).toBeGreaterThan(0);
  });

  test("bounds a legacy feature, whose positions are objects", () => {
    const sphere = getFeatureBoundingSphereFromBounds([
      {
        polygons: [
          {
            exterior: [
              { x: 139.7, y: 35.6, z: 0 },
              { x: 139.8, y: 35.6, z: 0 },
              { x: 139.8, y: 35.7, z: 12 },
            ],
          },
        ],
      },
    ]);

    expect(sphere).not.toBeNull();
    expect(sphere?.radius).toBeGreaterThan(0);
  });

  test("skips a position it cannot read rather than bounding NaN", () => {
    const sphere = getFeatureBoundingSphereFromBounds([
      { polygons: [{ exterior: [["a", "b"], [null], { x: 1 }] }] },
    ]);

    expect(sphere).toBeNull();
  });
});
