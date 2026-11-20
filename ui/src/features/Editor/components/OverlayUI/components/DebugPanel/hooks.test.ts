import { describe, expect, test } from "vitest";

import { toSelectedFeature } from "./hooks";

const select = (line: unknown, row = 0) => {
  const feature = toSelectedFeature(JSON.stringify(line), row);
  if (!feature) throw new Error("expected a feature");
  return feature;
};

describe("toSelectedFeature", () => {
  test("a line that is not valid JSON gives no feature", () => {
    expect(toSelectedFeature('{"id":1,"geometry":[1,', 0)).toBeNull();
  });

  test("carries the transformed geometry and the attributes", () => {
    const feature = select({
      id: "0195f3a0-0000-7000-8000-000000000005",
      attributes: { name: "Shibuya" },
      geometry: {
        Euclidean3D: {
          PolygonMesh: {
            frame: { Crs: 6697 },
            faces: [
              {
                exterior: [
                  [35.6, 139.7, 0],
                  [35.7, 139.7, 0],
                  [35.7, 139.8, 0],
                ],
                holes: [
                  [
                    [35.62, 139.72, 0],
                    [35.63, 139.72, 0],
                    [35.63, 139.73, 0],
                  ],
                ],
              },
            ],
          },
        },
      },
    });

    expect(feature._values.geometry.coordinates).toEqual([
      [
        [
          [139.7, 35.6, 0],
          [139.7, 35.7, 0],
          [139.8, 35.7, 0],
          [139.7, 35.6, 0],
        ],
        [
          [139.72, 35.62, 0],
          [139.72, 35.63, 0],
          [139.73, 35.63, 0],
          [139.72, 35.62, 0],
        ],
      ],
    ]);
    expect(feature._values.attributes).toEqual({ name: "Shibuya" });
    expect(feature.id).toBe('"0195f3a0-0000-7000-8000-000000000005"');
  });

  test("knows its line in the file, and what a view can draw of it", () => {
    const point = (frame: unknown) => ({
      id: "p",
      attributes: {},
      geometry: { Euclidean2D: { Point: { frame, position: [35.6, 139.7] } } },
    });

    const geographic = select(point({ Crs: 4326 }), 4);
    expect(geographic._row).toBe(4);
    expect(geographic._values.geometrySummary).toMatchObject({
      has2D: true,
      crs: [4326],
    });
    expect(select(point("Euclidean"))._values.geometrySummary).toMatchObject({
      hasPartWithoutCrs: true,
    });
  });

  test("an embedded texture becomes a description, not a wall of bytes", () => {
    const feature = select({
      id: "t",
      attributes: {},
      geometry: {
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
                          bytes: Array.from({ length: 200_000 }, () => 7),
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
      },
    });
    expect(JSON.stringify(feature).length).toBeLessThan(2000);
  });

  test("a legacy feature, and one without an id, still select", () => {
    const feature = select(
      {
        attributes: { a: 1 },
        geometry: {
          epsg: 4326,
          value: { flowGeometry2D: { point: { x: 3, y: 4 } } },
        },
      },
      7,
    );
    expect(feature.id).toBe("7");
    expect(feature._values.geometry).toMatchObject({ type: "Point" });
  });
});
