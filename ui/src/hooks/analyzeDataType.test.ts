import { describe, expect, test } from "vitest";

import { analyzeDataType } from "./useStreamingDebugRunQuery";

/** A new-format feature carrying one geometry. */
const next = (geometry: unknown) => ({ id: "1", attributes: {}, geometry });

/** A legacy-format feature, for the branch that still has to work. */
const legacy = (value: unknown, attributes: Record<string, unknown> = {}) => ({
  id: "1",
  attributes,
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

describe("new-format viewer selection", () => {
  test("2D geographic data opens the 2D map", () => {
    expect(analyzeDataType([next(point({ Crs: 4326 }))])).toMatchObject({
      geometryType: "Point (2D)",
      visualizerType: "2d-map",
    });
  });

  test("3D geographic data opens the globe", () => {
    expect(analyzeDataType([next(mesh({ Crs: 4979 }))])).toMatchObject({
      geometryType: "Triangle mesh (3D)",
      visualizerType: "3d-map",
    });
  });

  test("model-space 3D opens the model viewer, not a map", () => {
    // An OBJ or glTF read has no CRS, so the reader emits a Euclidean frame.
    // Those coordinates are not longitude and latitude, and a globe would put
    // them at null island.
    expect(analyzeDataType([next(mesh("Euclidean"))])).toMatchObject({
      visualizerType: "3d-model",
    });
  });

  test("a tangent plane follows the frame it is anchored in", () => {
    const anchored = (base: unknown) => ({
      Tangent: { base, origin: [0, 0, 0], u: [1, 0, 0], v: [0, 1, 0] },
    });

    expect(analyzeDataType([next(mesh(anchored("Euclidean")))])).toMatchObject({
      visualizerType: "3d-model",
    });
    expect(
      analyzeDataType([next(mesh(anchored({ Crs: 6697 })))]),
    ).toMatchObject({ visualizerType: "3d-map" });
  });

  test("descends into a collection to find what draws", () => {
    // The shape every CityGML feature takes.
    const cityGml = next({
      GeometryCollection: {
        members: [{ Euclidean3D: { ...mesh({ Crs: 6697 }).Euclidean3D } }],
        attrs: [{ lod: 2 }],
      },
    });

    expect(analyzeDataType([cityGml])).toMatchObject({
      visualizerType: "3d-map",
    });
  });

  test("offers no viewer for geometry with no GeoJSON form", () => {
    const cloud = next({
      Euclidean3D: {
        PointCloud: { frame: { Crs: 4979 }, segments: [] },
      },
    });

    expect(analyzeDataType([cloud])).toMatchObject({
      geometryType: "Point cloud",
      visualizerType: null,
    });
  });

  test("reports the predominant type, not whichever came first", () => {
    const features = [
      next(point({ Crs: 4326 })),
      next(mesh({ Crs: 4979 })),
      next(mesh({ Crs: 4979 })),
      next(mesh({ Crs: 4979 })),
    ];

    expect(analyzeDataType(features)).toMatchObject({
      geometryType: "Triangle mesh (3D)",
      visualizerType: "3d-map",
    });
  });

  test("ignores features with no geometry when choosing", () => {
    expect(
      analyzeDataType([next("None"), next(point({ Crs: 4326 }))]),
    ).toMatchObject({ visualizerType: "2d-map" });
  });

  test("offers nothing for an empty file", () => {
    expect(analyzeDataType([])).toEqual({
      geometryType: null,
      visualizerType: null,
    });
  });
});

describe("legacy viewer selection still works", () => {
  test("2D opens the 2D map, with a readable name", () => {
    expect(
      analyzeDataType([legacy({ flowGeometry2D: { point: { x: 1, y: 2 } } })]),
    ).toMatchObject({
      geometryType: "2D geometry",
      visualizerType: "2d-map",
    });
  });

  test("CityGML opens the globe", () => {
    expect(
      analyzeDataType([legacy({ cityGmlGeometry: { gmlGeometries: [] } })]),
    ).toMatchObject({
      geometryType: "CityGML geometry",
      visualizerType: "3d-map",
    });
  });

  test("3D with an OBJ source opens the model viewer", () => {
    expect(
      analyzeDataType([
        legacy(
          { flowGeometry3D: { point: { x: 1, y: 2, z: 3 } } },
          {
            source: "OBJ",
          },
        ),
      ]),
    ).toMatchObject({
      geometryType: "3D geometry",
      visualizerType: "3d-model",
    });
  });

  test("3D without one opens the globe", () => {
    expect(
      analyzeDataType([legacy({ flowGeometry3D: { point: { x: 1, y: 2 } } })]),
    ).toMatchObject({ visualizerType: "3d-map" });
  });
});
