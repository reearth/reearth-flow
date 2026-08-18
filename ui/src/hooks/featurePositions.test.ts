/**
 * The unit the debug panel's display budget counts in.
 *
 * `displayLimit` counts features, which says nothing about cost when one file's
 * feature is a point and another's is a 400-surface CityGML solid. Measured
 * against the transform's output a retained position is ~220 bytes, so 2000
 * features ranges from 17 MB to 867 MB — and the renderer process dies well
 * before the upper end. This is what bounds the second case.
 */
import { describe, expect, test } from "vitest";

import { transformNextFeature } from "@flow/utils/jsonl/transformNextFeature";

import { featurePositions } from "./useStreamingDebugRunQuery";

describe("featurePositions", () => {
  test("counts a position, a ring, and a mesh", () => {
    expect(
      featurePositions({ type: "Point", coordinates: [139.7, 35.6] }),
    ).toBe(1);
    expect(
      featurePositions({
        type: "LineString",
        coordinates: [
          [0, 0],
          [1, 1],
          [2, 2],
        ],
      }),
    ).toBe(3);
    expect(
      featurePositions({
        type: "Polygon",
        coordinates: [
          [
            [0, 0],
            [1, 0],
            [1, 1],
            [0, 0],
          ],
          [
            [0.2, 0.2],
            [0.3, 0.2],
            [0.3, 0.3],
            [0.2, 0.2],
          ],
        ],
      }),
    ).toBe(8);
  });

  test("adds up the members of a GeometryCollection", () => {
    expect(
      featurePositions({
        type: "GeometryCollection",
        geometries: [
          { type: "Point", coordinates: [0, 0] },
          {
            type: "LineString",
            coordinates: [
              [0, 0],
              [1, 1],
            ],
          },
        ],
      }),
    ).toBe(3);
  });

  test("survives a feature with no geometry, or an unconverted one", () => {
    expect(featurePositions(undefined)).toBe(0);
    expect(featurePositions(null)).toBe(0);
    // A point cloud gets a summary, with no coordinates at all.
    expect(
      featurePositions({ type: "Point cloud", summary: "Points: 12,000" }),
    ).toBe(0);
  });

  test("counts what a CityGML building actually costs", () => {
    const building = transformNextFeature({
      id: "x",
      attributes: {},
      geometry: {
        GeometryCollection: {
          members: [
            {
              Euclidean3D: {
                PolygonMesh: {
                  frame: { Crs: 6697 },
                  faces: Array.from({ length: 10 }, () => ({
                    exterior: [
                      [35.6, 139.7, 0],
                      [35.6, 139.8, 0],
                      [35.7, 139.8, 9],
                    ],
                  })),
                },
              },
            },
          ],
          attrs: [{ lod: 1 }],
        },
      },
    });

    // Ten faces, each closed to four positions. A feature count of 1 hides
    // that; this is the number the budget has to see.
    expect(featurePositions(building.geometry)).toBe(40);
  });
});
