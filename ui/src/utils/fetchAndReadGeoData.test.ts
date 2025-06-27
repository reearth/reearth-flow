import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

import { fetchAndReadData } from "./fetchAndReadGeoData";
import { parseJSONL } from "./jsonl";
import { intermediateDataTransform } from "./jsonl/transformIntermediateData";

vi.mock("./jsonl", () => ({
  parseJSONL: vi.fn(),
}));

vi.mock("./jsonl/transformIntermediateData", () => ({
  intermediateDataTransform: vi.fn(),
}));

describe("fetchAndReadData", () => {
  let fetchMock: any;

  beforeEach(() => {
    fetchMock = vi
      .spyOn(global, "fetch")
      .mockImplementation(() => Promise.resolve({} as Response));
    vi.spyOn(console, "error").mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("should return an error when URL is empty", async () => {
    const result = await fetchAndReadData("  ");
    expect(result).toEqual({
      fileContent: null,
      type: null,
      error: "Please enter a URL",
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("should handle failed fetch requests", async () => {
    fetchMock.mockResolvedValueOnce({
      ok: false,
      status: 404,
      statusText: "Not Found",
    });

    const result = await fetchAndReadData("https://example.com/data.geojson");
    expect(result).toEqual({
      fileContent: null,
      type: null,
      error: "Error fetching file: Failed to fetch: 404 Not Found",
    });
  });

  it("should fail if unsupported data", async () => {
    fetchMock.mockResolvedValueOnce({
      ok: true,
      text: () => Promise.resolve(),
    });
    const result = await fetchAndReadData("https://example.com/data.pdf");

    expect(result).toEqual({
      fileContent: null,
      type: null,
      error: "File format not supported",
    });
  });

  it("should parse GeoJSON files", async () => {
    const mockGeoJSON: GeoJSON.FeatureCollection = {
      type: "FeatureCollection",
      features: [
        {
          type: "Feature",
          properties: {
            name: "Shinjuku",
            population: 346000,
            ward: "Shinjuku-ku",
          },
          geometry: {
            type: "Point",
            coordinates: [139.7051, 35.6938],
          },
        },
      ],
    };
    fetchMock.mockResolvedValueOnce({
      ok: true,
      text: () => Promise.resolve(JSON.stringify(mockGeoJSON)),
    });

    const result = await fetchAndReadData("https://example.com/data.geojson");

    expect(result).toEqual({
      fileContent: mockGeoJSON,
      type: "geojson",
      error: null,
    });
  });

  it("should parse and transform JSONL files into GeoJson", async () => {
    const mockFeatures = [
      {
        type: "Feature",
        properties: {
          name: "Shinjuku",
          population: 346000,
          ward: "Shinjuku-ku",
        },
        geometry: {
          type: "Point",
          coordinates: [139.7051, 35.6938],
        },
      },
      {
        type: "Feature",
        properties: {
          name: "Shibuya",
          population: 346000,
          ward: "Shibuya-ku",
        },
        geometry: {
          type: "Point",
          coordinates: [139.702, 35.6581],
        },
      },
    ];

    const mockJSONLText = `{"id":"550e8400-e29b-41d4-a716-446655440000","attributes":{"name":"Shinjuku","population":346000,"ward":"Shinjuku-ku"},"geometry":{"epsg":4326,"value":{"FlowGeometry2D":{"point":{"x":139.7051,"y":35.6938,"z":null}}}},"metadata":{"feature_type":"Point","lod":null}}
{"id":"660e8400-e29b-41d4-a716-446655440001","attributes":{"name":"Shibuya","population":225000,"ward":"Shibuya-ku"},"geometry":{"epsg":4326,"value":{"FlowGeometry2D":{"point":{"x":139.7021,"y":35.6581,"z":null}}}},"metadata":{"feature_type":"Point","lod":null}}`;

    fetchMock.mockResolvedValueOnce({
      ok: true,
      text: () => Promise.resolve(mockJSONLText),
    });

    vi.mocked(parseJSONL).mockReturnValue(mockFeatures);

    const result = await fetchAndReadData("https://example.com/data.jsonl");

    const expectedResult = {
      fileContent: {
        type: "FeatureCollection",
        features: mockFeatures,
      },
      type: "geojson",
      error: null,
    };

    expect(parseJSONL).toHaveBeenCalledWith(mockJSONLText, {
      transform: intermediateDataTransform,
    });

    const parsedJSONL = parseJSONL(mockJSONLText, {
      transform: intermediateDataTransform,
    });

    const interData = {
      type: "FeatureCollection",
      features: parsedJSONL,
    };

    expect(interData).toEqual(expectedResult.fileContent);
    expect(result).toEqual(expectedResult);
  });
});
