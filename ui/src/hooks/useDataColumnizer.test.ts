import { renderHook } from "@testing-library/react";
import { describe, expect, test } from "vitest";

import { intermediateDataTransform } from "@flow/utils/jsonl/transformIntermediateData";

import useDataColumnizer from "./useDataColumnizer";

/** Run raw JSONL lines through the transform the streaming hook applies. */
function columnize(lines: unknown[]) {
  const features = lines.map(intermediateDataTransform);

  // Built once, outside the render callback: the hook re-runs its effect
  // whenever `parsedData` changes identity, and the real caller hands it a
  // memoized value.
  const parsedData = { type: "FeatureCollection", features };

  const { result } = renderHook(() =>
    useDataColumnizer({ parsedData, type: "geojson" }),
  );

  return {
    headers: (result.current.tableColumns as any[]).map((c) => c.header),
    rows: result.current.tableData as any[],
  };
}

describe("the row the raw dialog shows", () => {
  test("carries every attribute in the file, flat, nulls included", () => {
    const point = (id: string, attributes: Record<string, unknown>) => ({
      id,
      attributes,
      geometry: {
        Euclidean2D: {
          Point: { frame: { Crs: 4326 }, position: [35.6, 139.7] },
        },
      },
    });

    const { rows } = columnize([
      point("a", { name: "Shibuya", height: 12 }),
      point("b", { name: "Shinjuku" }), // no `height`
    ]);

    // Flat keys, not a nested `attributes` object.
    expect(rows[1]).not.toHaveProperty("attributes");
    expect(rows[1].attributesname).toBe('"Shinjuku"');
    // Present as null rather than missing, so the column stays readable.
    expect(rows[1].attributesheight).toBe("null");
  });
});

describe("new-format features reach the table", () => {
  test("a 2D feature contributes geometry and attribute columns", () => {
    const { headers, rows } = columnize([
      {
        id: "0195f3a0-0000-7000-8000-000000000001",
        attributes: { name: "Shibuya", gml_id: "bldg-1" },
        geometry: {
          Euclidean2D: {
            // Latitude first, as the engine writes it under EPSG:4326.
            Point: { frame: { Crs: 4326 }, position: [35.6, 139.7] },
          },
        },
      },
    ]);

    expect(headers).toEqual([
      "id",
      "geometry.type",
      "geometry.coordinates",
      "geometry.frame",
      "attributes.name",
      "attributes.gml_id",
    ]);
    expect(rows[0]).toMatchObject({
      geometrytype: '"Point"',
      geometrycoordinates: "[139.7,35.6]",
      geometryframe: '"EPSG:4326"',
      attributesname: '"Shibuya"',
    });
  });

  test("a 3D feature renders as geometry with its frame", () => {
    const { headers, rows } = columnize([
      {
        id: "0195f3a0-0000-7000-8000-000000000002",
        attributes: {},
        geometry: {
          Euclidean3D: {
            Solid: {
              frame: { Crs: 4979 },
              exterior: { PolygonMesh: { faces: [] } },
              interiors: [],
            },
          },
        },
      },
    ]);

    expect(headers).not.toContain("geometry.summary");
    expect(rows[0]).toMatchObject({
      geometrytype: '"MultiPolygon"',
      geometryframe: '"EPSG:4979"',
    });
  });

  test("an embedded texture becomes a description, not a wall of bytes", () => {
    const { rows } = columnize([
      {
        id: "0195f3a0-0000-7000-8000-000000000003",
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
      },
    ]);

    // The whole row, serialized, must not carry the 200k byte values.
    // `transformNextFeature` is what replaced them with a description; this
    // checks none of it leaked back in through a cell or through `_values`.
    expect(JSON.stringify(rows[0]).length).toBeLessThan(2000);
  });

  test("a feature with no geometry still lists its attributes", () => {
    const { headers, rows } = columnize([
      {
        id: "0195f3a0-0000-7000-8000-000000000004",
        attributes: { note: "attributes only" },
        geometry: "None",
      },
    ]);

    expect(headers).toEqual(["id", "attributes.note"]);
    expect(rows[0].attributesnote).toBe('"attributes only"');
  });

  test("columns are the union across a mixed-format file", () => {
    const { headers } = columnize([
      {
        id: "1",
        attributes: { a: 1 },
        geometry: {
          Euclidean2D: { Point: { frame: { Crs: 4326 }, position: [1, 2] } },
        },
      },
      {
        id: "2",
        attributes: { b: 2 },
        geometry: {
          epsg: 4326,
          value: { flowGeometry2D: { point: { x: 3, y: 4 } } },
        },
      },
    ]);

    expect(headers).toEqual(
      expect.arrayContaining([
        "geometry.type",
        "geometry.coordinates",
        "geometry.frame",
        "attributes.a",
        "attributes.b",
      ]),
    );
  });

  test("a row carries the values behind its cells, for the details panel", () => {
    const line = {
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
    };

    const features = [intermediateDataTransform(line)];
    const parsedData = { type: "FeatureCollection", features };
    const { result } = renderHook(() =>
      useDataColumnizer({ parsedData, type: "geojson" }),
    );

    const row = (result.current.tableData as any[])[0];

    // The cell string is cut to fit a cell; the value behind it is not, so the
    // details panel can format the coordinates it actually holds.
    // One face: its exterior ring, then its hole, each closed.
    expect(row._values.geometry.coordinates).toEqual([
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
    expect(row._values.attributes).toEqual({ name: "Shibuya" });

    // Underscored, so it does not become a column.
    expect(
      (result.current.tableColumns as any[]).map((c) => c.header),
    ).not.toContain("_values");
  });
});
