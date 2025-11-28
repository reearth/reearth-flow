import { ColumnDef } from "@tanstack/react-table";
import { useCallback, useEffect, useState } from "react";

import { SupportedDataTypes } from "@flow/hooks/useStreamingDebugRunQuery";

// Helper function to format cell values with truncation
function formatCellValue(value: any): string {
  if (value === undefined) return "-";

  const formatted = JSON.stringify(value);

  return formatted;
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
              }) as ColumnDef<any>,
          ),
        ];

        // Transform features for table display
        const tableData = features.map((feature: any, index: number) => ({
          id: JSON.stringify(feature.id || index),
          ...Object.fromEntries(
            Array.from(allGeometry).map((geometry) => {
              if (
                geometry === "coordinates" &&
                feature.geometry.type === "Polygon"
              ) {
                return [
                  `geometry${geometry}`,
                  formatCellValue(feature.geometry.coordinates || null),
                ];
              }
              if (
                geometry === "coordinates" &&
                feature.geometry.type === "MultiPolygon"
              ) {
                return [
                  `geometry${geometry}`,
                  formatCellValue(feature.geometry.coordinates || null),
                ];
              }
              if (
                geometry === "coordinates" &&
                feature.geometry.type === "LineString"
              ) {
                return [
                  `geometry${geometry}`,
                  formatCellValue(feature.geometry?.[geometry] || null),
                ];
              }
              return [
                `geometry${geometry}`,
                formatCellValue(feature.geometry?.[geometry] || null),
              ];
            }),
          ),
          ...Object.fromEntries(
            Array.from(allProps).map((prop) => [
              `attributes${prop}`,
              formatCellValue(feature.properties?.[prop] ?? null),
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
