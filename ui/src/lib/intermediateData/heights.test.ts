import { describe, expect, test } from "vitest";

import { geometryBounds, heightRange } from "./heights";

describe("heightRange", () => {
  test("spans every position's height", () => {
    expect(
      heightRange({
        type: "MultiPolygon",
        coordinates: [
          [
            [
              [139.7, 35.6, 52.1],
              [139.8, 35.6, 71.4],
              [139.8, 35.7, 60],
            ],
          ],
        ],
      }),
    ).toEqual({ min: 52.1, max: 71.4 });
  });

  test("reaches into a collection's members", () => {
    expect(
      heightRange({
        type: "GeometryCollection",
        geometries: [
          { type: "Point", coordinates: [139.7, 35.6, 3] },
          {
            type: "LineString",
            coordinates: [
              [139.7, 35.6, -2],
              [139.8, 35.6, 8],
            ],
          },
        ],
      }),
    ).toEqual({ min: -2, max: 8 });
  });

  test("a 2D geometry has none", () => {
    expect(
      heightRange({ type: "Point", coordinates: [139.7, 35.6] }),
    ).toBeNull();
    expect(heightRange(undefined)).toBeNull();
  });
});

describe("geometryBounds", () => {
  test("spans every position, with its heights", () => {
    expect(
      geometryBounds({
        type: "GeometryCollection",
        geometries: [
          { type: "Point", coordinates: [139.7, 35.6, 3] },
          {
            type: "LineString",
            coordinates: [
              [139.6, 35.8, 9],
              [139.9, 35.5, 5],
            ],
          },
        ],
      }),
    ).toEqual({
      west: 139.6,
      south: 35.5,
      east: 139.9,
      north: 35.8,
      heights: { min: 3, max: 9 },
    });
  });

  test("a 2D geometry has no heights, and nothing has no bounds", () => {
    expect(
      geometryBounds({ type: "Point", coordinates: [139.7, 35.6] })?.heights,
    ).toBeNull();
    expect(geometryBounds({ type: "Point", coordinates: [] })).toBeNull();
  });
});
