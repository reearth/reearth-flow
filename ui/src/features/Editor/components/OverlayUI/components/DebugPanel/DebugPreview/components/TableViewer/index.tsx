import { memo, useCallback } from "react";

import BasicBoiler from "@flow/components/BasicBoiler";
import { VirtualizedTable } from "@flow/components/visualizations/VirtualizedTable";
import useDataColumnizer from "@flow/hooks/useDataColumnizer";
import { useT } from "@flow/lib/i18n";
import { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

type Props = {
  fileContent: any | null;
  fileType: SupportedDataTypes | null;
  selectedFeature: any;
  onSingleClick?: (feature: any) => void;
  onDoubleClick?: (feature: any) => void;
  detectedGeometryType: string | null;
  totalFeatures: number;
};

const TableViewer: React.FC<Props> = memo(
  ({
    fileContent,
    fileType,
    selectedFeature,
    onSingleClick,
    onDoubleClick,
    detectedGeometryType,
    totalFeatures,
  }) => {
    const t = useT();

    // Use traditional columnizer for all data (streaming is now pre-transformed)
    const columnizer = useDataColumnizer({
      parsedData: fileContent,
      type: fileType,
    });

    // Handle row double click
    const handleRowDoubleClick = useCallback(
      (feature: any) => {
        onDoubleClick?.(feature);
      },
      [onDoubleClick],
    );

    // Loading state
    if (!fileContent || !columnizer.tableData) {
      return (
        <BasicBoiler text={t("Loading data...")} className="h-full" />
      );
    }

    // No data state
    if (!columnizer.tableData || columnizer.tableData.length === 0) {
      return (
        <BasicBoiler
          text={t("No data to display in table format")}
          className="h-full"
        />
      );
    }

    return (
      <div className="flex h-full flex-col">
        <div className="flex h-full flex-1 flex-col">
          {/* Table */}
          <div className="flex-1 overflow-hidden">
            <VirtualizedTable
              columns={columnizer.tableColumns}
              data={columnizer.tableData}
              selectColumns={true}
              showFiltering={true}
              condensed={true}
              selectedRow={selectedFeature}
              useStrictSelectedRow={true}
              onRowClick={onSingleClick}
              onRowDoubleClick={handleRowDoubleClick}
            />
          </div>

          {/* Status Display */}
          <div className="mt-1 flex items-center justify-between rounded-md bg-muted/50 px-3 py-1 text-xs text-muted-foreground">
            <div className="flex items-center gap-4">
              <span>
                {t("Rows")}:{" "}
                {(columnizer.tableData || []).length.toLocaleString()}
                {totalFeatures !== undefined &&
                  totalFeatures > 0 &&
                  ` / ${totalFeatures.toLocaleString()} ${t("total")}`}
              </span>
              {detectedGeometryType && (
                <span className="rounded bg-muted px-2 text-xs">
                  {detectedGeometryType}
                </span>
              )}
              <span>
                {t("Columns")}: {(columnizer.tableColumns || []).length}
              </span>
            </div>
          </div>
        </div>
      </div>
    );
  },
);

TableViewer.displayName = "TableViewer";

export default TableViewer;
