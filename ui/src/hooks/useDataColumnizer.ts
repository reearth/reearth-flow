import { ColumnDef } from "@tanstack/react-table";
import { useCallback, useEffect, useState } from "react";

import { Polygon, PolygonCoordinateRing } from "@flow/types/gisTypes/geoJSON";
import { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

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
          { accessorKey: "id", header: "id" },
          ...Array.from(allGeometry).map(
            (geometry) =>
              ({
                accessorKey: `geometry${geometry}`,
                header: `geometry.${geometry}`,
              }) as ColumnDef<any>,
          ),
          ...Array.from(allProps).map(
            (prop) =>
              ({
                accessorKey: `properties${prop}`,
                header: `properties.${prop}`,
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
                  simplifyPolygonCoordinates(feature.geometry),
                ];
              }
              return [
                `geometry${geometry}`,
                JSON.stringify(feature.geometry?.[geometry] || null),
              ];
            }),
          ),
          ...Object.fromEntries(
            Array.from(allProps).map((prop) => [
              `properties${prop}`,
              JSON.stringify(feature.properties?.[prop] || null),
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
    if (parsedData && !data) {
      handleDataLoaded();
    }
  }, [data, parsedData, type, handleDataLoaded]);

  return {
    tableData: data,
    tableColumns: columns,
  };
};

// simplifyPolygonCoordinates: Simplify GeoJSON Polygon coordinates for display. Output looks like this:
// [
//   [
//     [
//       [100, 0],
//       "...",
//       [100, 0]
//     ],
//     [
//       [100, 0],
//       "...",
//       [100, 0]
//     ]
//   ],
//   "...",
//   [
//     [
//       [100, 0],
//       "...",
//       [100, 0]
//     ],
//     [
//       [100, 0],
//       "...",
//       [100, 0]
//     ]
//   ]
// ]
function simplifyPolygonCoordinates(polygon: Polygon) {
  if (
    !polygon ||
    polygon.type !== "Polygon" ||
    !Array.isArray(polygon.coordinates)
  ) {
    throw new Error("Invalid GeoJSON Polygon");
  }

  const rings = polygon.coordinates;
  if (rings.length <= 4) {
    return rings.map((ring) => simplifyRing(ring));
  }

  const firstTwo = rings.slice(0, 2).map((ring) => simplifyRing(ring));
  const lastTwo = rings.slice(-2).map((ring) => simplifyRing(ring));

  return [...firstTwo, "...", ...lastTwo];
}

function simplifyRing(ring: PolygonCoordinateRing) {
  if (ring.length <= 4) {
    return ring; // Keep as is if 4 or fewer points
  }
  return JSON.stringify([ring[0], "...", ring[ring.length - 1]]);
}
