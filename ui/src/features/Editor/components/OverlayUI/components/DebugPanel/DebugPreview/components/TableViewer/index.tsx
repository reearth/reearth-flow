import { memo, useCallback, useState } from "react";

import BasicBoiler from "@flow/components/BasicBoiler";
import { VirtualizedTable } from "@flow/components/visualizations/VirtualizedTable";
import useDataColumnizer from "@flow/hooks/useDataColumnizer";
import { useT } from "@flow/lib/i18n";
import { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

import FeatureDetailsDialog from "./FeatureDetailsDialog";

type Props = {
  fileContent: any | null;
  fileType: SupportedDataTypes | null;
  selectedFeature: any;
  onSingleClick?: (feature: any) => void;
  onDoubleClick?: (feature: any) => void;

  // Streaming props
  isStreaming: boolean;
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

    // Streaming props
    isStreaming,
    detectedGeometryType,
    totalFeatures,
  }) => {
    const t = useT();

    console.log("Filecnte", fileContent);
    // Use traditional columnizer for all data (streaming is now pre-transformed)
    const columnizer = useDataColumnizer({
      parsedData: fileContent,
      type: fileType,
    });

    // Feature details dialog state
    const [selectedFeatureDetails, setSelectedFeatureDetails] = useState<
      any | null
    >(null);
    const [showDetailsDialog, setShowDetailsDialog] = useState(false);

    // Handle feature details dialog
    const handleShowDetails = useCallback((feature: any) => {
      setSelectedFeatureDetails(feature);
      setShowDetailsDialog(true);
    }, []);

    const handleRowDoubleClick = useCallback(
      (feature: any) => {
        handleShowDetails(feature);
        onDoubleClick?.(feature);
      },
      [handleShowDetails, onDoubleClick],
    );

    // Loading state
    if (isStreaming && (!fileContent || !columnizer.tableData)) {
      return (
        <BasicBoiler text={t("Loading streaming data...")} className="h-full" />
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

        {/* Feature details dialog */}
        <FeatureDetailsDialog
          feature={selectedFeatureDetails}
          open={showDetailsDialog}
          onOpenChange={setShowDetailsDialog}
        />
      </div>
    );
  },
);

TableViewer.displayName = "TableViewer";

export default TableViewer;
