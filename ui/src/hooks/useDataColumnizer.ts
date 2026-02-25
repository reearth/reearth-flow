import { ColumnDef } from "@tanstack/react-table";
import { useCallback, useEffect, useState } from "react";

import { SupportedDataTypes } from "@flow/hooks/useStreamingDebugRunQuery";

// Helper function to format cell values with truncation and to prevent large data from causing performance issues in the table

function formatCellValue(value: any): string {
  if (value === undefined) return "-";
  if (value === null) return "null";
  if (typeof value === "string") return JSON.stringify(value);
  if (Array.isArray(value)) {
    const items = value.slice(0, 5);
    return JSON.stringify(items);
  }
  if (value && typeof value === "object") {
    const keys = Object.keys(value);
    if (keys.length > 5) {
      const shownObject: Record<string, unknown> = {};
      keys.slice(0, 5).forEach((key) => {
        shownObject[key] = (value as any)[key];
      });
      const remainingCount = keys.length - 5;
      const suffix = remainingCount > 0 ? ` ... (+${remainingCount} keys)` : "";
      return JSON.stringify(shownObject) + suffix;
    }
    return JSON.stringify(value);
  }
  return String(value);
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
                cell: (info: any) => formatCellValue(info.getValue()),
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
                cell: (info: any) => formatCellValue(info.getValue()),
              }) as ColumnDef<any>,
          ),
        ];

        // Transform features for table display
        const tableData = features.map((feature: any, index: number) => ({
          id: JSON.stringify(feature.id || index),
          ...Object.fromEntries(
            Array.from(allGeometry).map((geometry) => [
              `geometry${geometry}`,
              feature.geometry?.[geometry] ?? null,
            ]),
          ),
          ...Object.fromEntries(
            Array.from(allProps).map((prop) => [
              `attributes${prop}`,
              feature.properties?.[prop] ?? null,
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
