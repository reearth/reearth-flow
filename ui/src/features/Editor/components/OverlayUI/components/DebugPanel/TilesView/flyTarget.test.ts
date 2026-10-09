import { describe, expect, test } from "vitest";

import { flyTargetOf } from "./flyTarget";

const row = (crs: number[], geometry: unknown) => ({
  _values: { geometry, geometrySummary: { crs } },
});

describe("flyTargetOf", () => {
  test("frames a row in a geographic CRS, heights included", () => {
    expect(
      flyTargetOf(
        row([6697], {
          type: "LineString",
          coordinates: [
            [139.7, 35.6, 50],
            [139.8, 35.7, 70],
          ],
        }),
      ),
    ).toEqual({
      west: 139.7,
      south: 35.6,
      east: 139.8,
      north: 35.7,
      minHeight: 50,
      maxHeight: 70,
    });
  });

  test("cannot place a row whose coordinates are not longitude and latitude", () => {
    expect(
      flyTargetOf(row([6677], { type: "Point", coordinates: [-12000, 34000] })),
    ).toBeUndefined();
    expect(
      flyTargetOf(row([], { type: "Point", coordinates: [1, 2] })),
    ).toBeUndefined();
  });
});
