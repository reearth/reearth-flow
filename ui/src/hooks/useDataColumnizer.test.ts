import { renderHook } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";

import { clearRasterStore } from "@flow/lib/intermediateData";
import { intermediateDataTransform } from "@flow/utils/jsonl/transformIntermediateData";

import useDataColumnizer from "./useDataColumnizer";

const OWNER = "https://example.test/node.out.jsonl.zst";

/** Run raw JSONL lines through the transform the streaming hook applies. */
function columnize(lines: unknown[]) {
  const features = lines.map((line, rowIndex) =>
    intermediateDataTransform(line, { owner: OWNER, rowIndex }),
  );

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

afterEach(() => clearRasterStore());

describe("new-format features reach the table", () => {
  test("a 2D feature contributes geometry and attribute columns", () => {
    const { headers, rows } = columnize([
      {
        id: "0195f3a0-0000-7000-8000-000000000001",
        attributes: { name: "Shibuya", gml_id: "bldg-1" },
        geometry: {
          Euclidean2D: {
            Point: { frame: { Crs: 4326 }, position: [139.7, 35.6] },
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

  test("a 3D feature shows its type and extent instead of coordinates", () => {
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

    expect(headers).toContain("geometry.summary");
    expect(rows[0]).toMatchObject({
      geometrytype: '"Solid"',
      geometryframe: '"EPSG:4979"',
      geometrysummary: '"1 exterior shell, 0 voids"',
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

    // The count reaches the table; the pixels do not.
    expect(rows[0].geometrytextures).toBe("1");
    for (const value of Object.values(rows[0])) {
      expect(String(value).length).toBeLessThan(200);
    }
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

  test("each row carries the line it came from, for view selection", () => {
    const { rows } = columnize([
      { id: "1", attributes: {}, geometry: "None" },
      { id: "2", attributes: {}, geometry: "None" },
    ]);

    expect(rows.map((row) => row._rowIndex)).toEqual([0, 1]);
  });
});
