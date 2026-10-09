import { describe, expect, test } from "vitest";

import { analyzeDataType } from "./useStreamingDebugRunQuery";

/** A new-format feature carrying one geometry. */
const next = (geometry: unknown) => ({ id: "1", attributes: {}, geometry });

/** A legacy-format feature, for the branch that still has to work. */
const legacy = (value: unknown) => ({
  id: "1",
  attributes: {},
  geometry: { epsg: 4326, value },
});

const point = (frame: unknown) => ({
  Euclidean2D: { Point: { frame, position: [35.6, 139.7] } },
});

const mesh = (frame: unknown) => ({
  Euclidean3D: {
    TriangularMesh: {
      frame,
      triangles: [
        [
          [0, 0, 0],
          [1, 0, 0],
          [1, 1, 0],
        ],
      ],
    },
  },
});

describe("new-format geometry label", () => {
  test("names the type the engine names", () => {
    expect(analyzeDataType([next(point({ Crs: 4326 }))])).toEqual({
      geometryType: "Point (2D)",
    });
    expect(analyzeDataType([next(mesh({ Crs: 4979 }))])).toEqual({
      geometryType: "Triangle mesh (3D)",
    });
  });

  test("names geometry with no GeoJSON form too", () => {
    const cloud = next({
      Euclidean3D: {
        PointCloud: { frame: { Crs: 4979 }, segments: [] },
      },
    });

    expect(analyzeDataType([cloud])).toEqual({ geometryType: "Point cloud" });
  });

  test("names the type only when the file agrees on one", () => {
    const uniform = [next(mesh({ Crs: 4979 })), next(mesh({ Crs: 4979 }))];

    expect(analyzeDataType(uniform)).toEqual({
      geometryType: "Triangle mesh (3D)",
    });
  });

  test("says Mixed rather than naming the most common type", () => {
    // A file of polylines, points and polygons is not a file of polylines.
    // Picking a winner would present a mixed file as uniform.
    const mixed = [
      next(point({ Crs: 4326 })),
      next(mesh({ Crs: 4979 })),
      next(mesh({ Crs: 4979 })),
      next(mesh({ Crs: 4979 })),
    ];

    expect(analyzeDataType(mixed)).toEqual({ geometryType: "Mixed" });
  });

  test("ignores features with no geometry", () => {
    expect(analyzeDataType([next("None"), next(point({ Crs: 4326 }))])).toEqual(
      { geometryType: "Point (2D)" },
    );
  });

  test("names nothing for an empty file", () => {
    expect(analyzeDataType([])).toEqual({ geometryType: null });
  });
});

describe("legacy geometry label", () => {
  test("gives each type a readable name", () => {
    expect(
      analyzeDataType([legacy({ flowGeometry2D: { point: { x: 1, y: 2 } } })]),
    ).toEqual({ geometryType: "2D geometry" });
    expect(
      analyzeDataType([legacy({ cityGmlGeometry: { gmlGeometries: [] } })]),
    ).toEqual({ geometryType: "CityGML geometry" });
    expect(
      analyzeDataType([legacy({ flowGeometry3D: { point: { x: 1, y: 2 } } })]),
    ).toEqual({ geometryType: "3D geometry" });
  });

  test("says Mixed for a legacy file too", () => {
    const mixed = [
      legacy({ flowGeometry2D: { point: { x: 1, y: 2 } } }),
      legacy({ cityGmlGeometry: { gmlGeometries: [] } }),
    ];

    expect(analyzeDataType(mixed)).toEqual({ geometryType: "Mixed" });
  });
});
