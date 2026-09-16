import { describe, expect, test } from "vitest";

import i18n from "@flow/lib/i18n/i18n";

import { intermediateDataTransform } from "./transformIntermediateData";
import { hasGeoJsonForm, transformNextFeature } from "./transformNextFeature";

const feature = (
  geometry: unknown,
  attributes: Record<string, unknown> = {},
) => ({
  id: "0195f3a0-0000-7000-8000-000000000000",
  attributes,
  geometry,
});

describe("2D geometry becomes GeoJSON", () => {
  test("point", () => {
    const result = transformNextFeature(
      feature({
        Euclidean2D: {
          Point: { frame: { Crs: 4326 }, position: [35.6, 139.7] },
        },
      }),
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
              { Point: { frame: { Crs: 4326 }, position: [2, 1] } },
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
    );

    expect(result.geometry).toMatchObject({
      type: "GeometryCollection",
      geometries: [{ type: "Point" }, { type: "LineString" }],
    });
  });

  test("swaps north-first CRS axes into GeoJSON longitude-first order", () => {
    // The engine stores coordinates in CRS authority order, which for
    // geographic CRSs is latitude first; GeoJSON is always longitude first.
    // Tokyo is 35.6N 139.7E.
    // One per family the engine supports: WGS84 2D/3D, JGD2011 geographic,
    // JGD2011 + height, and a plane-rectangular zone from each of the four
    // ranges (JGD2011, JGD2000, Tokyo datum, and JGD2011 + vertical height).
    for (const epsg of [4326, 4979, 6668, 6697, 6677, 2451, 30165, 10168]) {
      const result = transformNextFeature(
        feature({
          Euclidean2D: {
            Point: { frame: { Crs: epsg }, position: [35.6, 139.7] },
          },
        }),
      );

      expect(result.geometry).toMatchObject({ coordinates: [139.7, 35.6] });
    }
  });

  test("covers every CRS the engine supports, or names it east-first", () => {
    // The engine's supported set is the WKT1_ESRI table in
    // engine/runtime/action-sink/src/file/shapefile/crs.rs. Web Mercator is the
    // only entry that is genuinely east-first; everything else is a Japanese
    // geographic or plane-rectangular system and must be swapped. Without this,
    // a code the engine gains silently renders transposed.
    const EAST_FIRST = new Set([3857]);
    const engineSupported = [
      4326,
      4979,
      3857,
      6668,
      6697,
      ...Array.from({ length: 19 }, (_, i) => 2443 + i), // JGD2000 I-XIX
      ...Array.from({ length: 19 }, (_, i) => 6669 + i), // JGD2011 I-XIX
      ...Array.from({ length: 19 }, (_, i) => 30161 + i), // Tokyo I-XIX
      ...Array.from({ length: 13 }, (_, i) => 10162 + i), // JGD2011 + height I-XIII
    ];

    for (const epsg of engineSupported) {
      const result = transformNextFeature(
        feature({
          Euclidean2D: {
            Point: { frame: { Crs: epsg }, position: [35.6, 139.7] },
          },
        }),
      );

      expect(result.geometry).toMatchObject({
        coordinates: EAST_FIRST.has(epsg) ? [35.6, 139.7] : [139.7, 35.6],
      });
    }
  });

  test("leaves east-first and non-CRS frames in the order given", () => {
    const cases: [string, unknown][] = [
      ["web mercator", { Crs: 3857 }],
      ["UTM zone 54N", { Crs: 32654 }],
      ["euclidean", "Euclidean"],
      [
        "tangent plane, whose coords are in-plane metres",
        {
          Tangent: {
            base: { Crs: 6697 },
            origin: [0, 0, 0],
            u: [1, 0, 0],
            v: [0, 1, 0],
          },
        },
      ],
    ];

    for (const [, frame] of cases) {
      const result = transformNextFeature(
        feature({ Euclidean2D: { Point: { frame, position: [100, 200] } } }),
      );

      expect(result.geometry).toMatchObject({ coordinates: [100, 200] });
    }
  });

  test("swaps every coordinate of a ring, not just the first", () => {
    const result = transformNextFeature(
      feature({
        Euclidean2D: {
          Polygon: {
            frame: { Crs: 6697 },
            exterior: [
              [35.6, 139.7],
              [35.7, 139.8],
              [35.8, 139.9],
            ],
            z: 5,
          },
        },
      }),
    );

    expect(result.geometry).toMatchObject({
      coordinates: [
        [
          [139.7, 35.6, 5],
          [139.8, 35.7, 5],
          [139.9, 35.8, 5],
          [139.7, 35.6, 5],
        ],
      ],
    });
  });

  test("decides the swap per collection member, not per feature", () => {
    const result = transformNextFeature(
      feature({
        Euclidean2D: {
          Collection: {
            members: [
              { Point: { frame: { Crs: 4326 }, position: [35.6, 139.7] } },
              { Point: { frame: { Crs: 3857 }, position: [100, 200] } },
            ],
          },
        },
      }),
    );

    // Same member type, so the collection collapses to one MultiPoint — but
    // each member's frame still decides its own axis order.
    expect(result.geometry).toMatchObject({
      type: "MultiPoint",
      coordinates: [
        [139.7, 35.6],
        [100, 200],
      ],
    });
  });

  test("collapses a same-type collection so downstream reads coordinates", () => {
    const result = transformNextFeature(
      feature({
        Euclidean2D: {
          Collection: {
            members: [
              {
                LineString: {
                  frame: { Crs: 4326 },
                  coords: [
                    [35.6, 139.7],
                    [35.7, 139.8],
                  ],
                },
              },
              {
                LineString: {
                  frame: { Crs: 4326 },
                  coords: [
                    [35.8, 139.9],
                    [35.9, 140.0],
                  ],
                },
              },
            ],
          },
        },
      }),
    );

    // A GeoJSON MultiLineString arrives as a collection of polylines; keeping it
    // a GeometryCollection would hide the coordinates under `geometries`.
    expect(result.geometry).toMatchObject({
      type: "MultiLineString",
      coordinates: [
        [
          [139.7, 35.6],
          [139.8, 35.7],
        ],
        [
          [139.9, 35.8],
          [140.0, 35.9],
        ],
      ],
    });
  });

  test("concatenates parts when the members are already multi-part", () => {
    const face = (lat: number) => ({
      exterior: [
        [lat, 139.7],
        [lat, 139.8],
        [lat + 0.1, 139.8],
      ],
    });
    const result = transformNextFeature(
      feature({
        Euclidean2D: {
          Collection: {
            members: [
              { PolygonMesh: { frame: { Crs: 4326 }, faces: [face(35.6)] } },
              { PolygonMesh: { frame: { Crs: 4326 }, faces: [face(35.8)] } },
            ],
          },
        },
      }),
    );

    const geometry = result.geometry as {
      type: string;
      coordinates: unknown[];
    };
    expect(geometry.type).toBe("MultiPolygon");
    // Two polygons side by side, not two nested MultiPolygons.
    expect(geometry.coordinates).toHaveLength(2);
  });

  test("keeps GeometryCollection only when the members genuinely differ", () => {
    const result = transformNextFeature(
      feature({
        Euclidean2D: {
          Collection: {
            members: [
              { Point: { frame: { Crs: 4326 }, position: [35.6, 139.7] } },
              {
                LineString: {
                  frame: { Crs: 4326 },
                  coords: [
                    [35.6, 139.7],
                    [35.7, 139.8],
                  ],
                },
              },
            ],
          },
        },
      }),
    );

    expect(result.geometry).toMatchObject({ type: "GeometryCollection" });
  });

  test("takes a collection's frame from its members", () => {
    // A Collection has only `members` and `attrs` — no frame of its own — so
    // the frame has to come from the members. This is what a GeoPackage read
    // produces.
    const result = transformNextFeature(
      feature({
        Euclidean2D: {
          Collection: {
            members: [
              {
                Polygon: {
                  frame: { Crs: 6668 },
                  exterior: [
                    [35.659, 139.675],
                    [35.659, 139.676],
                    [35.66, 139.676],
                  ],
                },
              },
            ],
          },
        },
      }),
    );

    expect(result.geometry).toMatchObject({
      type: "MultiPolygon",
      frame: "EPSG:6668",
    });
  });

  test("reports every frame a collection's members sit in", () => {
    // Members may differ; naming one would claim they agree.
    const result = transformNextFeature(
      feature({
        Euclidean2D: {
          Collection: {
            members: [
              { Point: { frame: { Crs: 6668 }, position: [35.6, 139.7] } },
              { Point: { frame: { Crs: 3857 }, position: [100, 200] } },
            ],
          },
        },
      }),
    );

    expect(result.geometry).toMatchObject({
      frame: "EPSG:6668, EPSG:3857",
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
    );

    expect(result.geometry).toMatchObject({
      frame: "Tangent plane (EPSG:4979)",
    });
  });
});

describe("3D geometry becomes a summary", () => {
  test("flattens a 3D mesh to a MultiPolygon", () => {
    const result = transformNextFeature(
      feature({
        Euclidean3D: {
          PolygonMesh: {
            frame: { Crs: 4979 },
            faces: [{ exterior: [] }, { exterior: [] }, { exterior: [] }],
          },
        },
      }),
    );

    expect(result.geometry).toMatchObject({
      type: "MultiPolygon",
      frame: "EPSG:4979",
    });
    // Converted geometry carries no invented descriptors.
    expect(result.geometry).not.toHaveProperty("summary");
  });

  test("renders a 3D point with its altitude, axes swapped", () => {
    const result = transformNextFeature(
      feature({
        Euclidean3D: {
          Point: { frame: { Crs: 4979 }, position: [35.6, 139.7, 40] },
        },
      }),
    );

    expect(result.geometry).toMatchObject({
      type: "Point",
      coordinates: [139.7, 35.6, 40],
    });
  });

  test("renders a 3D polyline, which is what a GeoJSON reader emits", () => {
    const result = transformNextFeature(
      feature({
        Euclidean3D: {
          LineString: {
            frame: { Crs: 4979 },
            coords: [
              [35.6, 139.7, 10],
              [35.7, 139.8, 20],
            ],
          },
        },
      }),
    );

    expect(result.geometry).toMatchObject({
      type: "LineString",
      coordinates: [
        [139.7, 35.6, 10],
        [139.8, 35.7, 20],
      ],
    });
  });

  test("flattens a solid's shells into one MultiPolygon", () => {
    const result = transformNextFeature(
      feature({
        Euclidean3D: {
          Solid: {
            frame: { Crs: 4979 },
            exterior: {
              PolygonMesh: {
                faces: [
                  {
                    exterior: [
                      [35.6, 139.7, 0],
                      [35.7, 139.7, 0],
                      [35.7, 139.8, 0],
                    ],
                  },
                ],
              },
            },
            interiors: [
              {
                TriangularMesh: {
                  triangles: [
                    [
                      [35.6, 139.7, 5],
                      [35.7, 139.7, 5],
                      [35.7, 139.8, 5],
                    ],
                  ],
                },
              },
            ],
          },
        },
      }),
    );

    const geometry = result.geometry as {
      type: string;
      coordinates: number[][][][];
    };
    expect(geometry.type).toBe("MultiPolygon");
    // One polygon from the exterior shell, one from the void.
    expect(geometry.coordinates).toHaveLength(2);
    // Shells are frameless and borrow the solid's frame, so they swap too.
    expect(geometry.coordinates[0][0][0]).toEqual([139.7, 35.6, 0]);
    expect(geometry.coordinates[1][0][0]).toEqual([139.7, 35.6, 5]);
  });

  test("renders a 3D collection so a multi-geometry feature still draws", () => {
    const result = transformNextFeature(
      feature({
        Euclidean3D: {
          Collection: {
            members: [
              { Point: { frame: { Crs: 4979 }, position: [35.6, 139.7, 1] } },
              {
                LineString: {
                  frame: { Crs: 4979 },
                  coords: [
                    [35.6, 139.7, 1],
                    [35.7, 139.8, 2],
                  ],
                },
              },
            ],
          },
        },
      }),
    );

    expect(result.geometry).toMatchObject({
      type: "GeometryCollection",
      geometries: [
        { type: "Point", coordinates: [139.7, 35.6, 1] },
        { type: "LineString" },
      ],
    });
  });

  test("keeps a summary for the geometry with no GeoJSON form", () => {
    const cloud = transformNextFeature(
      feature({
        Euclidean3D: {
          PointCloud: {
            frame: { Crs: 4979 },
            segments: [{ positions: { F64: [[0, 0, 0]] } }],
          },
        },
      }),
    );
    expect(cloud.geometry).toEqual({
      type: "Point cloud",
      frame: "EPSG:4979",
      summary: "Points: 1",
    });

    const csg = transformNextFeature(
      feature({ Euclidean3D: { Csg: { Union: [] } } }),
    );
    expect(csg.geometry).toMatchObject({ type: "Boolean combination" });
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
    );

    expect(result.geometry).toMatchObject({
      type: "Point cloud",
      summary: "Points: 3",
    });
  });

  test("names the boolean operation of a CSG geometry", () => {
    const result = transformNextFeature(
      feature({ Euclidean3D: { Csg: { Difference: [] } } }),
    );

    expect(result.geometry).toMatchObject({
      type: "Boolean combination",
      summary: "Difference",
    });
  });

  test("follows the UI language rather than being English-only", async () => {
    const cloud = () =>
      transformNextFeature(
        feature({
          Euclidean3D: {
            PointCloud: {
              frame: { Crs: 4979 },
              segments: [
                {
                  positions: {
                    F64: Array.from({ length: 12_000 }, () => [0, 0, 0]),
                  },
                },
              ],
            },
          },
        }),
      ).geometry as { summary: string };

    // English grouping, from the key itself — the catalogue leaves these empty.
    expect(cloud().summary).toBe("Points: 12,000");

    i18n.addResource("ja", "translation", "Points: {{n}}", "点群: {{n}}");
    await i18n.changeLanguage("ja");
    try {
      // The label translates, and the count is grouped for the language it is
      // read in — `12.000` and `12,000` are different numbers.
      expect(cloud().summary).toBe("点群: 12,000");

      // Spanish groups with a dot, and is one of the app's languages, so the
      // number has to follow the UI rather than the browser.
      await i18n.changeLanguage("es");
      expect(cloud().summary).toBe("Points: 12.000");
    } finally {
      await i18n.changeLanguage("en");
    }
  });

  test("replaces an inline texture's bytes with a note of what it was", () => {
    const parsed = feature({
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
    });

    transformNextFeature(parsed);

    const raster = (parsed.geometry as any).Euclidean3D.Polygon.appearance
      .materials[0].Pbr.base_color_map.raster;
    expect(raster.InMemory).toEqual({
      mime_type: "image/png",
      byteLength: 4,
    });
  });

  test("leaves an external texture's uri alone", () => {
    const parsed = feature({
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
    });

    transformNextFeature(parsed);

    expect(
      (parsed.geometry as any).Euclidean3D.Polygon.appearance.materials[0].Phong
        .diffuse_map.raster,
    ).toEqual({ Uri: "file:///t.jpg" });
  });

  test("does not walk into coordinate bulk looking for images", () => {
    // A ring long enough that a naive walk would be obvious in a profile, and
    // whose numbers must survive untouched.
    const exterior = Array.from({ length: 50_000 }, (_, i) => [i, i, 0]);
    const parsed = feature({
      Euclidean3D: { Polygon: { frame: { Crs: 4979 }, exterior } },
    });

    transformNextFeature(parsed);

    expect((parsed.geometry as any).Euclidean3D.Polygon.exterior).toHaveLength(
      50_000,
    );
  });
});

describe("features without drawable geometry", () => {
  test("an absent geometry yields attributes only", () => {
    const result = transformNextFeature(
      feature("None", { name: "attributes only" }),
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
    );

    expect(result.geometry).toMatchObject({ type: "Geometry collection" });
  });
});

/**
 * The shape every CityGML feature takes under new-geometry: a top-level
 * GeometryCollection, one member per `lodN`, with the level in the parallel
 * `attrs` (citygml_parser/pipeline.rs).
 */
function cityGmlFeature(levels: number[]) {
  const solid = (lat: number) => ({
    Euclidean3D: {
      Solid: {
        frame: { Crs: 6697 },
        exterior: {
          PolygonMesh: {
            faces: [
              {
                exterior: [
                  [lat, 139.7, 0],
                  [lat, 139.8, 0],
                  [lat + 0.01, 139.8, 0],
                ],
              },
            ],
          },
        },
      },
    },
  });

  return feature({
    GeometryCollection: {
      members: levels.map((_, index) => solid(35.6 + index * 0.1)),
      attrs: levels.map((level) => ({ lod: level })),
    },
  });
}

describe("a CityGML feature's per-LOD collection", () => {
  test("draws, rather than falling back to a summary", () => {
    const result = transformNextFeature(cityGmlFeature([2]));

    // Before, a top-level collection was never converted and the map got
    // nothing at all.
    expect(result.geometry).toMatchObject({
      type: "MultiPolygon",
      frame: "EPSG:6697",
    });
  });

  test("draws one level of detail, not all of them stacked", () => {
    const result = transformNextFeature(cityGmlFeature([1, 2, 3]));

    const geometry = result.geometry as {
      coordinates: unknown[];
      lod: number;
    };
    // LOD1, matching the legacy CityGML renderer's preference.
    expect(geometry.lod).toBe(1);
    // One member's worth of polygons, not three.
    expect(geometry.coordinates).toHaveLength(1);
  });

  test("falls back to the highest level available", () => {
    const result = transformNextFeature(cityGmlFeature([2, 3]));

    expect(result.geometry).toMatchObject({ lod: 2 });
  });

  test("skips a footprint in favour of a solid", () => {
    // LOD0 is a footprint — one flat surface at ground level. Preferring it
    // because it is the smallest draws a city as flat polygons instead of as
    // buildings, and with heights normalized to the ground they are all but
    // invisible. The legacy renderer asked for LOD1 first, and so does this.
    expect(
      transformNextFeature(cityGmlFeature([0, 1, 2])).geometry,
    ).toMatchObject({ lod: 1 });
    expect(transformNextFeature(cityGmlFeature([0, 2])).geometry).toMatchObject(
      {
        lod: 2,
      },
    );
  });

  test("holds the finest level for the on-select upgrade", () => {
    // The map draws LOD1 and swaps in the finest level when a feature is
    // selected, as legacy did — legacy could, because it held the engine's
    // whole record. This holds the one extra level that needs, and no more.
    const result = transformNextFeature(cityGmlFeature([0, 1, 2, 3]));

    expect(result.geometry).toMatchObject({ lod: 1 });
    expect(result.lodDetail).toMatchObject({ lod: 3 });
    expect((result.lodDetail?.geometry as { type: string }).type).toBe(
      "MultiPolygon",
    );
  });

  test("holds nothing extra when the drawn level is already the finest", () => {
    expect(transformNextFeature(cityGmlFeature([1])).lodDetail).toBeUndefined();
    expect(transformNextFeature(cityGmlFeature([2])).lodDetail).toBeUndefined();
  });

  test("keeps the finer level off the geometry, so it is not a column", () => {
    // `useDataColumnizer` builds a column per geometry key; a second blob of
    // coordinates there is noise in the table.
    const result = transformNextFeature(cityGmlFeature([1, 2]));

    expect(result.geometry).not.toHaveProperty("lodDetail");
    expect(result.lodDetail).toBeDefined();
  });

  test("still draws a file that has nothing but footprints", () => {
    // Nothing preferred is present, so the lowest declared level is drawn
    // rather than nothing at all.
    expect(transformNextFeature(cityGmlFeature([0])).geometry).toMatchObject({
      lod: 0,
    });
  });

  test("swaps member axes through the collection", () => {
    const result = transformNextFeature(cityGmlFeature([1]));

    const geometry = result.geometry as { coordinates: number[][][][] };
    // EPSG:6697 is north-first, so the member's [lat, lon, z] comes out
    // longitude-first even though the swap is decided inside the member.
    expect(geometry.coordinates[0][0][0]).toEqual([139.7, 35.6, 0]);
  });

  test("draws every member when none declares a level", () => {
    const result = transformNextFeature(
      feature({
        GeometryCollection: {
          members: [
            {
              Euclidean3D: {
                Point: { frame: { Crs: 6697 }, position: [35.6, 139.7, 1] },
              },
            },
            {
              Euclidean3D: {
                Point: { frame: { Crs: 6697 }, position: [35.7, 139.8, 2] },
              },
            },
          ],
        },
      }),
    );

    expect(result.geometry).toMatchObject({ type: "MultiPoint" });
    expect(result.geometry).not.toHaveProperty("lod");
  });

  test("finds the frame through a nested collection", () => {
    // CityGML reaches two levels: a GeometryCollection of per-LOD members,
    // each of which may be a MultiSurface and so a collection in turn. No
    // collection carries a frame, so it has to come from the leaves.
    const result = transformNextFeature(
      feature({
        GeometryCollection: {
          members: [
            {
              Euclidean3D: {
                Collection: {
                  members: [
                    {
                      Polygon: {
                        frame: { Crs: 6697 },
                        exterior: [
                          [35.6, 139.7, 0],
                          [35.6, 139.8, 0],
                          [35.7, 139.8, 0],
                        ],
                      },
                    },
                  ],
                },
              },
            },
          ],
          attrs: [{ lod: 2 }],
        },
      }),
    );

    expect(result.geometry).toMatchObject({
      type: "MultiPolygon",
      frame: "EPSG:6697",
      lod: 2,
    });
  });

  test("carries appearance through as the renderer's colour fields", () => {
    // Legacy got `materials` and `polygonMaterials` for free, by passing the
    // engine's record through. The new model keeps appearance per leaf, so it
    // is projected onto the same two fields — otherwise every building falls
    // back to the roof/wall/floor heuristic and a coloured file looks wrong.
    const result = transformNextFeature(
      feature({
        Euclidean3D: {
          PolygonMesh: {
            frame: { Crs: 6697 },
            faces: [
              {
                exterior: [
                  [35.6, 139.7, 0],
                  [35.6, 139.8, 0],
                  [35.7, 139.8, 9],
                ],
              },
              {
                exterior: [
                  [35.6, 139.7, 0],
                  [35.6, 139.8, 0],
                  [35.7, 139.8, 3],
                ],
              },
            ],
            appearance: {
              materials: [
                {
                  Phong: {
                    diffuse: [0.8, 0.2, 0.1],
                    transparency: 0.25,
                    specular: [0, 0, 0],
                    emissive: [0, 0, 0],
                    ambient_intensity: 0.2,
                    shininess: 0.2,
                  },
                },
              ],
              themes: [{ theme: "rgbTexture", front: { PerFace: [0, null] } }],
              default_theme: "rgbTexture",
            },
          },
        },
      }),
    );

    expect(result.geometry).toMatchObject({
      materials: [{ diffuseColor: [0.8, 0.2, 0.1], transparency: 0.25 }],
      // One entry per surface; the second face is unbound.
      polygonMaterials: [0, null],
    });
  });

  test("reads the default theme's binding, not just the first", () => {
    const result = transformNextFeature(
      feature({
        Euclidean3D: {
          Polygon: {
            frame: { Crs: 6697 },
            exterior: [
              [35.6, 139.7, 0],
              [35.6, 139.8, 0],
              [35.7, 139.8, 0],
            ],
            appearance: {
              materials: [
                { Phong: { diffuse: [1, 0, 0], transparency: 0 } },
                { Pbr: { base_color: [0, 0, 1, 0.5] } },
              ],
              themes: [
                { theme: "other", front: { Uniform: 0 } },
                { theme: "chosen", front: { Uniform: 1 } },
              ],
              default_theme: "chosen",
            },
          },
        },
      }),
    );

    // The PBR material, and its alpha read as CityGML transparency.
    expect(result.geometry).toMatchObject({
      polygonMaterials: [1],
      materials: [
        { diffuseColor: [1, 0, 0] },
        { diffuseColor: [0, 0, 1], transparency: 0.5 },
      ],
    });
  });

  test("rebases material indices when members are merged", () => {
    // Each member indexes its own palette; concatenating the palettes without
    // shifting the indices would colour the second member from the first's.
    const member = (diffuse: number[]) => ({
      Euclidean3D: {
        Polygon: {
          frame: { Crs: 6697 },
          exterior: [
            [35.6, 139.7, 0],
            [35.6, 139.8, 0],
            [35.7, 139.8, 0],
          ],
          appearance: {
            materials: [{ Phong: { diffuse, transparency: 0 } }],
            themes: [{ theme: "t", front: { Uniform: 0 } }],
            default_theme: "t",
          },
        },
      },
    });

    const result = transformNextFeature(
      feature({
        GeometryCollection: {
          members: [member([1, 0, 0]), member([0, 1, 0])],
          attrs: [{ lod: 2 }, { lod: 2 }],
        },
      }),
    );

    expect(result.geometry).toMatchObject({
      polygonMaterials: [0, 1],
      materials: [{ diffuseColor: [1, 0, 0] }, { diffuseColor: [0, 1, 0] }],
    });
  });

  test("keeps unbound surfaces aligned when only some members are painted", () => {
    const bare = {
      Euclidean3D: {
        Polygon: {
          frame: { Crs: 6697 },
          exterior: [
            [35.6, 139.7, 0],
            [35.6, 139.8, 0],
            [35.7, 139.8, 0],
          ],
        },
      },
    };
    const painted = {
      Euclidean3D: {
        Polygon: {
          frame: { Crs: 6697 },
          exterior: [
            [35.6, 139.7, 0],
            [35.6, 139.8, 0],
            [35.7, 139.8, 0],
          ],
          appearance: {
            materials: [{ Phong: { diffuse: [0, 0, 1], transparency: 0 } }],
            themes: [{ theme: "t", front: { Uniform: 0 } }],
            default_theme: "t",
          },
        },
      },
    };

    const result = transformNextFeature(
      feature({
        GeometryCollection: {
          members: [bare, painted],
          attrs: [{ lod: 2 }, { lod: 2 }],
        },
      }),
    );

    // The bare member still advances the binding by its own surface, or the
    // painted one's index would land on the wrong surface.
    expect(result.geometry).toMatchObject({ polygonMaterials: [null, 0] });
  });

  test("emits no colour fields when a leaf carries no appearance", () => {
    const result = transformNextFeature(cityGmlFeature([1]));

    expect(result.geometry).not.toHaveProperty("materials");
    expect(result.geometry).not.toHaveProperty("polygonMaterials");
  });

  test("still summarizes a collection with nothing drawable in it", () => {
    const result = transformNextFeature(
      feature({ GeometryCollection: { members: ["None", "None"] } }),
    );

    expect(result.geometry).toMatchObject({ type: "Geometry collection" });
  });
});

describe("geometry the schema does not know", () => {
  test("names it by its discriminant instead of reporting Unknown", () => {
    const result = transformNextFeature(
      feature({
        Euclidean3D: { Nurbs: { frame: { Crs: 6697 }, degree: 3 } },
      }),
    );

    // A newer engine can emit a variant this build has never seen; the table
    // should still say what it was.
    expect(result.geometry).toEqual({
      type: "Nurbs",
      frame: "EPSG:6697",
    });
  });
});

describe("hasGeoJsonForm", () => {
  test("agrees with what the transform actually converts", () => {
    for (const variant of [
      "Point",
      "LineString",
      "Polygon",
      "PolygonMesh",
      "TriangularMesh",
      "Solid",
      "Collection",
    ]) {
      expect(hasGeoJsonForm(variant)).toBe(true);
    }

    for (const variant of ["PointCloud", "Csg", "Hyperbola", null]) {
      expect(hasGeoJsonForm(variant)).toBe(false);
    }
  });
});

describe("coordinate sharing", () => {
  test("reuses the source arrays when no reordering is needed", () => {
    // An east-first frame needs no reordering, so it must not pay an
    // allocation per position on a file that is nothing but positions.
    const coords = [
      [100, 200],
      [300, 400],
    ];
    const result = transformNextFeature(
      feature({
        Euclidean2D: { LineString: { frame: { Crs: 3857 }, coords } },
      }),
    );

    const emitted = (result.geometry as { coordinates: number[][] })
      .coordinates;
    expect(emitted[0]).toBe(coords[0]);
  });

  test("copies rather than mutating when the axes are swapped", () => {
    const coords = [[35.6, 139.7]];
    const result = transformNextFeature(
      feature({
        Euclidean2D: { LineString: { frame: { Crs: 4326 }, coords } },
      }),
    );

    const emitted = (result.geometry as { coordinates: number[][] })
      .coordinates;
    expect(emitted[0]).toEqual([139.7, 35.6]);
    // The parsed record is the caller's; a swap copies rather than reorders
    // it in place.
    expect(coords[0]).toEqual([35.6, 139.7]);
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
        Euclidean2D: { Point: { frame: { Crs: 4326 }, position: [2, 1] } },
      }),
    );

    expect(result.geometry).toMatchObject({
      type: "Point",
      coordinates: [1, 2],
      frame: "EPSG:4326",
    });
  });
});
