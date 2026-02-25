import { ColumnDef } from "@tanstack/react-table";
import { useCallback, useEffect, useState } from "react";

import { SupportedDataTypes } from "@flow/hooks/useStreamingDebugRunQuery";

// Fully serialize a value to a string for use as the accessor value.
// Keeping the full string ensures global filtering can match any part of the data.
function serializeValue(value: any): string {
  if (value === undefined) return "-";
  if (value === null) return "null";
  return JSON.stringify(value);
}

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
  const [columns, setColumns] = useState<ColumnDef<any>[]>([]);

  const handleDataLoaded = useCallback(() => {
    if (type === "geojson") {
      // Extract features and their properties from GeoJSON
      const features = parsedData.features || [];
      if (features.length > 0) {
        // Get unique properties from all geometries
        const allGeometry = new Set<string>();
        features.forEach((feature: any) => {
          if (feature.geometry) {
            Object.keys(feature.geometry).forEach((key) =>
              allGeometry.add(key),
            );
          }
        });
        // Get unique properties from all features
        const allProps = new Set<string>();
        features.forEach((feature: any) => {
          if (feature.properties) {
            Object.keys(feature.properties).forEach((key) => allProps.add(key));
          }
        });

        // Create columns for table
        const tableColumns: ColumnDef<any>[] = [
          {
            accessorKey: "id",
            header: "id",
            size: 200,
            maxSize: 400,
            minSize: 100,
          },
          ...Array.from(allGeometry).map(
            (geometry) =>
              ({
                accessorKey: `geometry${geometry}`,
                header: `geometry.${geometry}`,
                size: 200,
                maxSize: 400,
                minSize: 100,
                cell: (info: any) => truncateDisplayValue(info.getValue()),
              }) as ColumnDef<any>,
          ),
          ...Array.from(allProps).map(
            (prop) =>
              ({
                accessorKey: `attributes${prop}`,
                header: `attributes.${prop}`,
                size: 200,
                maxSize: 400,
                minSize: 100,
                cell: (info: any) => truncateDisplayValue(info.getValue()),
              }) as ColumnDef<any>,
          ),
        ];

        // Store fully serialized strings as accessor values so global filtering
        // can match any part of the data. Truncation happens only in the cell renderer.
        const tableData = features.map((feature: any, index: number) => ({
          id: JSON.stringify(feature.id || index),
          ...Object.fromEntries(
            Array.from(allGeometry).map((geometry) => [
              `geometry${geometry}`,
              serializeValue(feature.geometry?.[geometry] ?? null),
            ]),
          ),
          ...Object.fromEntries(
            Array.from(allProps).map((prop) => [
              `attributes${prop}`,
              serializeValue(feature.properties?.[prop] ?? null),
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
