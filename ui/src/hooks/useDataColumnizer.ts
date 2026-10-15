import { useCallback, useEffect, useState } from "react";

import type { SupportedDataTypes } from "@flow/hooks/useStreamingDebugRunQuery";
import type { AppColumnDef } from "@flow/lib/table/features";
import { safeSerialize } from "@flow/utils/valueSummary";

// Truncate a pre-serialized string for display only, to prevent large payloads
// from degrading render performance.
const DISPLAY_MAX_CHARS = 100;
function truncateDisplayValue(str: string): string {
  if (!str) return "";
  if (str.length <= DISPLAY_MAX_CHARS) return str;
  return str.slice(0, DISPLAY_MAX_CHARS) + "…";
}

export default ({
  parsedData,
  type,
}: {
  parsedData?: any;
  type?: SupportedDataTypes | null;
}) => {
  const [data, setData] = useState<any>(null);
  const [columns, setColumns] = useState<AppColumnDef<any>[]>([]);

  const handleDataLoaded = useCallback(() => {
    if (type === "geojson") {
      // Extract features and their properties from GeoJSON
      const features = parsedData.features || [];
      if (features.length > 0) {
        // The table shows what a feature says: its id and attributes. Its
        // geometry is for the details view, which reads it from `_values`.
        // Get unique properties from all features
        const allProps = new Set<string>();
        features.forEach((feature: any) => {
          if (feature.properties) {
            Object.keys(feature.properties).forEach((key) => allProps.add(key));
          }
        });

        // Create columns for table
        const tableColumns: AppColumnDef<any>[] = [
          {
            accessorKey: "id",
            header: "Feature ID",
            size: 200,
            maxSize: 400,
            minSize: 100,
          },
          ...Array.from(allProps).map(
            (prop) =>
              ({
                accessorKey: `attributes${prop}`,
                header: prop,
                size: 200,
                maxSize: 400,
                minSize: 100,
                cell: (info: any) => truncateDisplayValue(info.getValue()),
              }) as AppColumnDef<any>,
          ),
        ];

        // Store serialized strings as accessor values so global filtering can
        // match any part of the data; values too large to serialize whole are
        // stored as a bounded preview of their leading content. Truncation
        // happens only in the cell renderer.
        const tableData = features.map((feature: any, index: number) => ({
          id: JSON.stringify(feature.id || index),
          // The values behind the serialized columns, for the details panel to
          // format properly — a cell string is cut to fit a cell, and past
          // `LARGE_VALUE_THRESHOLD` it is a bounded preview that no longer
          // parses back. Underscored, so it does not become a column. These are
          // references to objects the parsed feature already holds, so nothing
          // is retained that was not retained already.
          _values: {
            geometry: feature.geometry ?? {},
            attributes: feature.properties ?? {},
            geometrySummary: feature.geometrySummary,
          },
          // The feature's line in its file, which is the row the API renders.
          // It travels with the row, so it survives sorting and searching.
          _row: index,
          ...Object.fromEntries(
            Array.from(allProps).map((prop) => [
              `attributes${prop}`,
              safeSerialize(feature.properties?.[prop] ?? null),
            ]),
          ),
        }));

        setColumns(tableColumns);
        setData(tableData);
      }
    }
    // } else if (type === 'csv') {
    //   // For CSV, the data is already in tabular format
    //   if (parsedData.length > 0) {
    //     const firstRow = parsedData[0];
    //     const tableColumns = Object.keys(firstRow).map(key => ({
    //       field: key,
    //       headerName: key,
    //       width: 150
    //     }));

    //     setColumns(tableColumns);
    //     setData(parsedData);
    //   }
    // }
  }, [parsedData, type]);

  useEffect(() => {
    if (parsedData) {
      handleDataLoaded();
    }
  }, [parsedData, handleDataLoaded]);

  return {
    tableData: data,
    tableColumns: columns,
  };
};
