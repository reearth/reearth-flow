import { describe, expect, test } from "vitest";

import {
  hasNonWgs84Geometry,
  hasUnsupportedCrs,
  isValidWgs84Coordinates,
  isValidWgs84Position,
} from "./wgs84";

// Real values from a JGD2011 / Japan Plane Rectangular CS III (EPSG:6671)
// dataset: metres, which Cesium would read as degrees.
const PROJECTED_POSITION = [32709.0026, -178466.4974];
const WGS84_POSITION = [132.52239, 34.390707];

describe("isValidWgs84Position", () => {
  test("accepts lon/lat within range, including the bounds", () => {
    expect(isValidWgs84Position(...(WGS84_POSITION as [number, number]))).toBe(
      true,
    );
    expect(isValidWgs84Position(0, 0)).toBe(true);
    expect(isValidWgs84Position(-180, -90)).toBe(true);
    expect(isValidWgs84Position(180, 90)).toBe(true);
  });

  test("rejects projected metres", () => {
    expect(
      isValidWgs84Position(...(PROJECTED_POSITION as [number, number])),
    ).toBe(false);
  });

  test("rejects out-of-range, non-finite and non-numeric values", () => {
    expect(isValidWgs84Position(181, 0)).toBe(false);
    expect(isValidWgs84Position(0, 91)).toBe(false);
    expect(isValidWgs84Position(NaN, 0)).toBe(false);
    expect(isValidWgs84Position(0, Infinity)).toBe(false);
    expect(isValidWgs84Position("132", 34)).toBe(false);
    expect(isValidWgs84Position(132, undefined)).toBe(false);
  });
});

describe("isValidWgs84Coordinates", () => {
  test("validates positions at any nesting depth", () => {
    expect(isValidWgs84Coordinates(WGS84_POSITION)).toBe(true);
    expect(isValidWgs84Coordinates([WGS84_POSITION, [132.6, 34.4]])).toBe(true);
    expect(isValidWgs84Coordinates([[WGS84_POSITION, [132.6, 34.4]]])).toBe(
      true,
    );
  });

  test("rejects the whole member when a single position is invalid", () => {
    expect(
      isValidWgs84Coordinates([[WGS84_POSITION, PROJECTED_POSITION]]),
    ).toBe(false);
  });

  test("rejects empty and non-array members", () => {
    expect(isValidWgs84Coordinates([])).toBe(false);
    expect(isValidWgs84Coordinates(null)).toBe(false);
    expect(isValidWgs84Coordinates(undefined)).toBe(false);
  });
});

describe("hasNonWgs84Geometry", () => {
  test("passes a WGS84 GeoJSON geometry", () => {
    expect(
      hasNonWgs84Geometry({
        type: "Polygon",
        coordinates: [[WGS84_POSITION, [132.6, 34.4], [132.6, 34.5]]],
      }),
    ).toBe(false);
  });

  test("flags a projected GeoJSON geometry", () => {
    expect(
      hasNonWgs84Geometry({
        type: "Polygon",
        coordinates: [[PROJECTED_POSITION, [32700.5126, -178464.4974]]],
      }),
    ).toBe(true);
  });

  test("flags projected CityGML {x, y, z} vertices", () => {
    expect(
      hasNonWgs84Geometry({
        type: "CityGmlGeometry",
        gmlGeometries: [
          {
            polygons: [
              {
                exterior: [
                  { x: 32709.0026, y: -178466.4974, z: 12 },
                  { x: 32700.5126, y: -178464.4974, z: 12 },
                ],
              },
            ],
          },
        ],
      }),
    ).toBe(true);
  });

  test("passes WGS84 CityGML {x, y, z} vertices", () => {
    expect(
      hasNonWgs84Geometry({
        type: "CityGmlGeometry",
        gmlGeometries: [
          {
            polygons: [
              {
                exterior: [
                  { x: 132.52239, y: 34.390707, z: 12 },
                  { x: 132.52241, y: 34.390709, z: 12 },
                ],
              },
            ],
          },
        ],
      }),
    ).toBe(false);
  });

  test("stops after the sample limit rather than walking everything", () => {
    // First position is valid, the bad one sits past the limit.
    const coordinates = [
      Array.from({ length: 10 }, () => WGS84_POSITION).concat([
        PROJECTED_POSITION,
      ]),
    ];

    expect(hasNonWgs84Geometry({ coordinates }, 5)).toBe(false);
    expect(hasNonWgs84Geometry({ coordinates }, 100)).toBe(true);
  });

  test("treats missing geometry as renderable rather than broken", () => {
    expect(hasNonWgs84Geometry(undefined)).toBe(false);
    expect(hasNonWgs84Geometry(null)).toBe(false);
    expect(hasNonWgs84Geometry({})).toBe(false);
  });
});

describe("hasUnsupportedCrs", () => {
  const ring = (positions: number[][]) => ({
    type: "Polygon",
    coordinates: [positions],
  });

  test("keeps a WGS84 GeoJSON feature", () => {
    expect(
      hasUnsupportedCrs(ring([WGS84_POSITION, [132.6, 34.4], [132.6, 34.5]])),
    ).toBe(false);
  });

  test("rejects a projected GeoJSON feature", () => {
    expect(hasUnsupportedCrs(ring([PROJECTED_POSITION]))).toBe(true);
  });

  test("checks GeoJSON exhaustively, past the CRS sample limit", () => {
    // The regression this guards: a single bad vertex buried deep in a long
    // ring is exactly what blows up Cesium's rhumb subdivision, and the
    // sampled CRS check stops looking after 500 positions.
    const positions = Array.from({ length: 2000 }, () => WGS84_POSITION);
    positions[1500] = PROJECTED_POSITION;

    expect(hasNonWgs84Geometry(ring(positions))).toBe(false); // sampled: misses it
    expect(hasUnsupportedCrs(ring(positions))).toBe(true); // exhaustive: catches it
  });

  test("catches a NaN vertex past the sample limit", () => {
    const positions = Array.from({ length: 1000 }, () => WGS84_POSITION);
    positions[900] = [NaN, 34.39];

    expect(hasUnsupportedCrs(ring(positions))).toBe(true);
  });

  test("samples payloads that carry no `coordinates` member", () => {
    const cityGml = {
      type: "CityGmlGeometry",
      gmlGeometries: [
        {
          polygons: [{ exterior: [{ x: 32709.0026, y: -178466.4974, z: 12 }] }],
        },
      ],
    };

    expect(hasUnsupportedCrs(cityGml)).toBe(true);
  });

  test("treats a feature without geometry as renderable", () => {
    expect(hasUnsupportedCrs(undefined)).toBe(false);
    expect(hasUnsupportedCrs(null)).toBe(false);
  });
});
