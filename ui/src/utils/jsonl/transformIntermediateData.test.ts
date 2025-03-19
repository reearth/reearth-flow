import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

import { intermediateDataTransform } from "./transformIntermediateData";

describe("intermediateDataTransform", () => {
  // Spy on console.warn
  beforeEach(() => {
    vi.spyOn(console, "warn").mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("should return the original data if no geometry is present", () => {
    const data = { id: "123", attributes: { name: "Test" } };
    expect(intermediateDataTransform(data)).toEqual(data);
  });

  it("should return the original data with a warning for unknown geometry type", () => {
    const data = {
      id: "123",
      attributes: { name: "Test" },
      geometry: {
        value: {
          someUnknownGeometry: {},
        },
      },
    };

    const result = intermediateDataTransform(data);

    expect(result).toEqual(data);
    expect(console.warn).toHaveBeenCalledWith(
      "Unknown geometry type detected. Displaying raw data.",
    );
  });

  it("should return the original data with a warning for 3D geometry", () => {
    const data = {
      id: "123",
      attributes: { name: "Test" },
      geometry: {
        value: {
          flowGeometry3D: {},
        },
      },
    };

    const result = intermediateDataTransform(data);

    expect(result).toEqual(data);
    expect(console.warn).toHaveBeenCalledWith(
      "3D geometry detected, but 3D viewer is not supported yet. Displaying raw data.",
    );
  });

  describe("2D geometry transformation", () => {
    it("should transform point geometry correctly", () => {
      const data = {
        id: "123",
        attributes: { name: "Test Point" },
        geometry: {
          value: {
            flowGeometry2D: {
              point: { x: 10, y: 20 },
            },
          },
        },
      };

      const expected = {
        id: "123",
        type: "Feature",
        properties: { name: "Test Point" },
        geometry: {
          type: "Point",
          coordinates: [10, 20],
        },
      };

      expect(intermediateDataTransform(data)).toEqual(expected);
    });

    it("should transform polygon geometry correctly", () => {
      const data = {
        id: "123",
        attributes: { name: "Test Polygon" },
        geometry: {
          value: {
            flowGeometry2D: {
              polygon: {
                exterior: [
                  { x: 0, y: 0 },
                  { x: 10, y: 0 },
                  { x: 10, y: 10 },
                  { x: 0, y: 10 },
                  { x: 0, y: 0 },
                ],
                interiors: [],
              },
            },
          },
        },
      };

      const expected = {
        id: "123",
        type: "Feature",
        properties: { name: "Test Polygon" },
        geometry: {
          type: "Polygon",
          coordinates: [
            [
              [0, 0],
              [10, 0],
              [10, 10],
              [0, 10],
              [0, 0],
            ],
          ],
        },
      };

      expect(intermediateDataTransform(data)).toEqual(expected);
    });

    it("should transform polygon with interiors correctly", () => {
      const data = {
        id: "123",
        attributes: { name: "Test Polygon with Hole" },
        geometry: {
          value: {
            flowGeometry2D: {
              polygon: {
                exterior: [
                  { x: 0, y: 0 },
                  { x: 10, y: 0 },
                  { x: 10, y: 10 },
                  { x: 0, y: 10 },
                  { x: 0, y: 0 },
                ],
                interiors: [
                  [
                    { x: 0, y: 0 },
                    { x: 10, y: 0 },
                    { x: 10, y: 10 },
                    { x: 0, y: 10 },
                    { x: 0, y: 0 },
                  ],
                  [
                    { x: 1, y: 1 },
                    { x: 10, y: 0 },
                    { x: 10, y: 10 },
                    { x: 0, y: 10 },
                    { x: 1, y: 1 },
                  ],
                ],
              },
            },
          },
        },
      };

      const expected = {
        id: "123",
        type: "Feature",
        properties: { name: "Test Polygon with Hole" },
        geometry: {
          type: "Polygon",
          coordinates: [
            [
              [0, 0],
              [10, 0],
              [10, 10],
              [0, 10],
              [0, 0],
            ],
            [
              [
                [0, 0],
                [10, 0],
                [10, 10],
                [0, 10],
                [0, 0],
              ],
              [
                [1, 1],
                [10, 0],
                [10, 10],
                [0, 10],
                [1, 1],
              ],
            ],
          ],
        },
      };

      expect(intermediateDataTransform(data)).toEqual(expected);
    });

    it("should transform lineString geometry correctly", () => {
      const data = {
        id: "123",
        attributes: { name: "Test LineString" },
        geometry: {
          value: {
            flowGeometry2D: {
              lineString: [
                { x: 0, y: 0 },
                { x: 10, y: 10 },
                { x: 20, y: 0 },
              ],
            },
          },
        },
      };

      const expected = {
        id: "123",
        type: "Feature",
        properties: { name: "Test LineString" },
        geometry: {
          type: "LineString",
          coordinates: [
            [0, 0],
            [10, 10],
            [20, 0],
          ],
        },
      };

      expect(intermediateDataTransform(data)).toEqual(expected);
    });

    it("should transform multiPoint geometry correctly", () => {
      const data = {
        id: "123",
        attributes: { name: "Test MultiPoint" },
        geometry: {
          value: {
            flowGeometry2D: {
              multiPoint: [
                { x: 0, y: 0 },
                { x: 10, y: 10 },
                { x: 20, y: 0 },
              ],
            },
          },
        },
      };

      const expected = {
        id: "123",
        type: "Feature",
        properties: { name: "Test MultiPoint" },
        geometry: {
          type: "MultiPoint",
          coordinates: [
            [0, 0],
            [10, 10],
            [20, 0],
          ],
        },
      };

      expect(intermediateDataTransform(data)).toEqual(expected);
    });

    it("should transform multiPolygon geometry correctly", () => {
      const data = {
        id: "123",
        attributes: { name: "Test MultiPolygon" },
        geometry: {
          value: {
            flowGeometry2D: {
              multiPolygon: [
                {
                  exterior: [
                    { x: 0, y: 0 },
                    { x: 10, y: 0 },
                    { x: 10, y: 10 },
                    { x: 0, y: 10 },
                    { x: 0, y: 0 },
                  ],
                  interiors: [],
                },
                {
                  exterior: [
                    { x: 20, y: 20 },
                    { x: 30, y: 20 },
                    { x: 30, y: 30 },
                    { x: 20, y: 30 },
                    { x: 20, y: 20 },
                  ],
                  interiors: [
                    [
                      [22, 22],
                      [28, 22],
                      [28, 28],
                      [22, 28],
                      [22, 22],
                    ],
                  ],
                },
              ],
            },
          },
        },
      };

      const expected = {
        id: "123",
        type: "Feature",
        properties: { name: "Test MultiPolygon" },
        geometry: {
          type: "MultiPolygon",
          coordinates: [
            [
              [
                [0, 0],
                [10, 0],
                [10, 10],
                [0, 10],
                [0, 0],
              ],
            ],
            [
              [
                [20, 20],
                [30, 20],
                [30, 30],
                [20, 30],
                [20, 20],
              ],
              [
                [22, 22],
                [28, 22],
                [28, 28],
                [22, 28],
                [22, 22],
              ],
            ],
          ],
        },
      };

      expect(intermediateDataTransform(data)).toEqual(expected);
    });

    it("should transform multiLineString geometry correctly", () => {
      const data = {
        id: "123",
        attributes: { name: "Test MultiLineString" },
        geometry: {
          value: {
            flowGeometry2D: {
              multiLineString: [
                [
                  { x: 0, y: 0 },
                  { x: 10, y: 10 },
                ],
                [
                  { x: 20, y: 20 },
                  { x: 30, y: 30 },
                ],
              ],
            },
          },
        },
      };

      const expected = {
        id: "123",
        type: "Feature",
        properties: { name: "Test MultiLineString" },
        geometry: {
          type: "MultiLineString",
          coordinates: [
            [
              [0, 0],
              [10, 10],
            ],
            [
              [20, 20],
              [30, 30],
            ],
          ],
        },
      };

      expect(intermediateDataTransform(data)).toEqual(expected);
    });

    it("should return the original geometry if no recognized type is found", () => {
      const unknownGeometry = {
        unknownType: {
          data: [1, 2, 3],
        },
      };

      const data = {
        id: "123",
        attributes: { name: "Test Unknown" },
        geometry: {
          value: {
            flowGeometry2D: unknownGeometry,
          },
        },
      };

      const expected = {
        id: "123",
        type: "Feature",
        properties: { name: "Test Unknown" },
        geometry: unknownGeometry,
      };

      expect(intermediateDataTransform(data)).toEqual(expected);
    });
  });
});
