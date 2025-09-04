import { ColumnDef } from "@tanstack/react-table";
import { useCallback, useState } from "react";

type StreamingTableData = {
  rows: any[];
  columns: ColumnDef<any>[];
  totalRows: number;
  knownColumns: Set<string>;
};

type UseStreamingDataColumnizerOptions = {
  /** Maximum number of rows to keep in memory */
  maxRows?: number;
  /** Whether to auto-expand columns as new properties are discovered */
  autoExpandColumns?: boolean;
};

export const DEFAULT_CELL_VALUE_MAX_LENGTH = 100;

export const useStreamingDataColumnizer = (
  options: UseStreamingDataColumnizerOptions = {},
) => {
  const { maxRows = 50000, autoExpandColumns = true } = options;

  const [tableData, setTableData] = useState<StreamingTableData>({
    rows: [],
    columns: [],
    totalRows: 0,
    knownColumns: new Set<string>(),
  });

  const extractColumns = useCallback(
    (features: any[], currentKnownColumns: Set<string>) => {
      const newColumns = new Set(currentKnownColumns);

      features.forEach((feature) => {
        // Add standard columns
        newColumns.add("id");

        // Handle different geometry types
        if (feature.geometry) {
          // For Flow geometries, extract properties based on type
          if (feature.geometry.value) {
            const geometryValue = feature.geometry.value;

            if (geometryValue.FlowGeometry2D || geometryValue.flowGeometry2D) {
              newColumns.add("geometry.type");
              newColumns.add("geometry.coordinates");
              newColumns.add("geometry.epsg");
            } else if (
              geometryValue.FlowGeometry3D ||
              geometryValue.flowGeometry3D
            ) {
              newColumns.add("geometry.type");
              newColumns.add("geometry.coordinates");
              newColumns.add("geometry.epsg");
            } else if (geometryValue.CityGmlGeometry) {
              newColumns.add("geometry.gml_geometries");
              newColumns.add("geometry.materials");
              newColumns.add("geometry.textures");
            }
          } else {
            // Standard GeoJSON geometry
            Object.keys(feature.geometry).forEach((key) => {
              newColumns.add(`geometry.${key}`);
            });
          }
        }

        // Add attribute columns
        if (feature.attributes) {
          Object.keys(feature.attributes).forEach((key) => {
            newColumns.add(`attributes.${key}`);
          });
        }

        // Add properties columns (for GeoJSON compatibility)
        if (feature.properties) {
          Object.keys(feature.properties).forEach((key) => {
            newColumns.add(`properties.${key}`);
          });
        }
      });

      return newColumns;
    },
    [],
  );

  const createTableColumns = useCallback(
    (columnNames: Set<string>): ColumnDef<any>[] => {
      return Array.from(columnNames)
        .sort()
        .map((columnName) => {
          // Convert column name to match traditional columnizer format (remove all dots)
          const accessorKey = columnName.replace(/\./g, "");
          return {
            accessorKey,
            header: columnName,
            size: 200, // Default column width
            maxSize: 400, // Maximum column width
            minSize: 100, // Minimum column width
            cell: ({ row }) => {
              const value = row.original[accessorKey];
              return formatCellValue(value);
            },
          };
        });
    },
    [],
  );

  const transformFeaturesForTable = useCallback(
    (features: any[], columns: Set<string>) => {
      return features.map((feature, index) => {
        const row: any = {};

        columns.forEach((columnName) => {
          // Convert column name to match traditional columnizer format (remove all dots)
          const key = columnName.replace(/\./g, "");
          const value = getNestedValue(feature, columnName);
          // Store both formatted (for display) and original (for copying) values
          row[key] = formatCellValue(value);
          row[`${key}_original`] = value; // Store original value for copying
        });

        // Ensure we have an ID
        if (!row.id) {
          row.id = JSON.stringify(feature.id || index);
        }

        return row;
      });
    },
    [],
  );

  const addBatch = useCallback(
    (newFeatures: any[]) => {
      if (newFeatures.length === 0) return;

      setTableData((prev) => {
        // Extract new columns if auto-expand is enabled
        const updatedColumns = autoExpandColumns
          ? extractColumns(newFeatures, prev.knownColumns)
          : prev.knownColumns;

        // Only recreate table columns if new columns were discovered
        const needsColumnUpdate =
          updatedColumns.size !== prev.knownColumns.size;
        const tableColumns = needsColumnUpdate
          ? createTableColumns(updatedColumns)
          : prev.columns;

        // Note: We allow adding features beyond maxRows when user explicitly requests "Load More"
        // The maxRows limit is enforced by the slice operation below

        // Transform new features to table rows
        const newRows = transformFeaturesForTable(newFeatures, updatedColumns);

        // Combine with existing rows, respecting max rows limit
        const combinedRows = [...prev.rows, ...newRows];
        const limitedRows = maxRows
          ? combinedRows.slice(-maxRows) // Keep most recent rows
          : combinedRows;

        return {
          rows: limitedRows,
          columns: tableColumns,
          totalRows: prev.totalRows + newFeatures.length,
          knownColumns: updatedColumns,
        };
      });
    },
    [
      autoExpandColumns,
      extractColumns,
      createTableColumns,
      transformFeaturesForTable,
      maxRows,
    ],
  );

  const reset = useCallback(() => {
    setTableData({
      rows: [],
      columns: [],
      totalRows: 0,
      knownColumns: new Set(),
    });
  }, []);

  return {
    // Table data
    tableData: tableData.rows,
    tableColumns: tableData.columns,
    totalRows: tableData.totalRows,

    // Statistics
    displayedRows: tableData.rows.length,
    knownColumnCount: tableData.knownColumns.size,

    // Control functions
    addBatch,
    reset,
  };
};

// Helper function to get nested property values
function getNestedValue(obj: any, path: string): any {
  const parts = path.split(".");
  let current = obj;

  for (const part of parts) {
    if (current == null) return null;
    current = current[part];
  }

  return current;
}

// Helper function to format cell values for display
function formatCellValue(value: any, maxLength = DEFAULT_CELL_VALUE_MAX_LENGTH): string {
  if (value == null) return "";

  let formatted: string;

  // Handle coordinate arrays specially
  if (Array.isArray(value)) {
    // For GeoJSON coordinates, show simplified version
    if (value.length > 0 && Array.isArray(value[0])) {
      if (value.length <= 4) {
        formatted = JSON.stringify(value);
      } else {
        // Simplify long coordinate arrays
        formatted = JSON.stringify([value[0], "...", value[value.length - 1]]);
      }
    } else {
      formatted = JSON.stringify(value);
    }
  } else if (typeof value === "object") {
    formatted = JSON.stringify(value);
  } else {
    formatted = String(value);
  }

  // Truncate long content with ellipsis
  if (formatted.length > maxLength) {
    return formatted.substring(0, maxLength - 3) + "...";
  }

  return formatted;
}
