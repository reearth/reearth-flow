import { renderHook } from "@testing-library/react";
import { describe, expect, test } from "vitest";

import { intermediateDataTransform } from "@flow/utils/jsonl/transformIntermediateData";

import useDataColumnizer from "./useDataColumnizer";

/** Run raw JSONL lines through the transform the streaming hook applies. */
function columnize(lines: unknown[]) {
  const features = lines.map((line) => {
    const transformed = intermediateDataTransform(line);
    // The streaming hook attaches the parsed record for raw inspection; do the
    // same here, so what reaches the table is what reaches it in the app.
    transformed.source = line;
    return transformed;
  });

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

    // The whole row, serialized, must not carry the 200k byte values —
    // `_source` holds the parsed record, which the transform strips in place.
    expect(JSON.stringify(rows[0]).length).toBeLessThan(2000);
    expect(JSON.stringify(rows[0])).toContain("byteLength");
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

  test("a row carries the engine's own record for raw inspection", () => {
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
                // Dropped by the derived GeoJSON; must survive here.
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
    features[0].source = line;
    const parsedData = { type: "FeatureCollection", features };
    const { result } = renderHook(() =>
      useDataColumnizer({ parsedData, type: "geojson" }),
    );

    const row = (result.current.tableData as any[])[0];
    const source = row._source as typeof line;

    // The engine's nested shape, not the flattened GeoJSON projection.
    expect(source.geometry.Euclidean3D.PolygonMesh.faces[0].holes).toHaveLength(
      1,
    );
    expect(source.geometry.Euclidean3D.PolygonMesh.frame).toEqual({
      Crs: 6697,
    });
    // Underscored, so it does not become a column.
    expect(
      (result.current.tableColumns as any[]).map((c) => c.header),
    ).not.toContain("_source");
  });
});
