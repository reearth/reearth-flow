import { describe, expect, test } from "vitest";

import { summarizeGeometry } from "./summary";

const point2D = (frame: unknown) => ({
  Euclidean2D: { Point: { frame, position: [34.744, 137.4626] } },
});

const solid = (frame: unknown) => ({
  Euclidean3D: {
    Solid: {
      frame,
      exterior: {
        PolygonMesh: {
          faces: [
            {
              exterior: [
                [34.7486, 137.4597, 86.7],
                [34.7486, 137.4595, 86.7],
                [34.7487, 137.4595, 86.7],
              ],
            },
          ],
        },
      },
    },
  },
});

describe("summarizeGeometry", () => {
  test("a 2D feature in a CRS", () => {
    expect(summarizeGeometry(point2D({ Crs: 4326 }))).toMatchObject({
      has2D: true,
      has3D: false,
      crs: [4326],
      hasPartWithoutCrs: false,
      has3DWithCrs: false,
    });
  });

  test("a building: 3D parts in a collection", () => {
    const building = {
      GeometryCollection: {
        members: [solid({ Crs: 6697 }), solid({ Crs: 6697 })],
      },
    };

    expect(summarizeGeometry(building)).toMatchObject({
      has2D: false,
      has3D: true,
      crs: [6697],
      has3DWithCrs: true,
    });
  });

  test("a collection holding both embeddings, in different CRSs", () => {
    const mixed = {
      GeometryCollection: {
        members: [point2D({ Crs: 4326 }), solid({ Crs: 6697 })],
      },
    };

    expect(summarizeGeometry(mixed)).toMatchObject({
      has2D: true,
      has3D: true,
      crs: [4326, 6697],
    });
  });

  test("3D parts with no CRS give a 3D model nothing to draw", () => {
    expect(summarizeGeometry(solid("Euclidean"))).toMatchObject({
      has3D: true,
      crs: [],
      hasPartWithoutCrs: true,
      has3DWithCrs: false,
    });
  });

  test("a part on a tangent plane is placed by the plane's anchor", () => {
    const tangent = {
      Tangent: { base: { Crs: 6697 }, origin: [0, 0, 0] },
    };

    expect(summarizeGeometry(solid(tangent))).toMatchObject({
      crs: [6697],
      hasPartWithoutCrs: false,
      has3DWithCrs: true,
    });
  });

  test("names the type the engine names", () => {
    expect(summarizeGeometry(point2D({ Crs: 4326 })).label).toBe("Point (2D)");
  });

  test("no geometry at all", () => {
    expect(summarizeGeometry("None")).toEqual({
      label: "",
      has2D: false,
      has3D: false,
      crs: [],
      hasPartWithoutCrs: false,
      has3DWithCrs: false,
      hasSurface: false,
      hasNonSurface: false,
    });
  });

  test("tells surfaces from the parts 3D Tiles leave out", () => {
    expect(summarizeGeometry(solid({ Crs: 6697 }))).toMatchObject({
      hasSurface: true,
      hasNonSurface: false,
    });
    expect(summarizeGeometry(point2D({ Crs: 4326 }))).toMatchObject({
      hasSurface: false,
      hasNonSurface: true,
    });
    const both = {
      GeometryCollection: {
        members: [point2D({ Crs: 4326 }), solid({ Crs: 6697 })],
      },
    };
    expect(summarizeGeometry(both)).toMatchObject({
      hasSurface: true,
      hasNonSurface: true,
    });
  });
});
