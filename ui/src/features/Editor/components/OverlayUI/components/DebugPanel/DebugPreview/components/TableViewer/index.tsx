import { memo, useCallback } from "react";

import BasicBoiler from "@flow/components/BasicBoiler";
import { VirtualizedTable } from "@flow/components/visualizations/VirtualizedTable";
import useDataColumnizer from "@flow/hooks/useDataColumnizer";
import { useT } from "@flow/lib/i18n";

import FeatureDetailsOverlay from "./FeatureDetailsOverlay";

type Props = {
  fileContent: any | null;
  selectedFeature: any;
  onSingleClick?: (feature: any) => void;
  onDoubleClick?: (feature: any) => void;
  detectedGeometryType: string | null;
  totalFeatures: number;
  detailsOverlayOpen: boolean;
  detailsFeature: any;
  formattedData: ReturnType<typeof useDataColumnizer>;
  onCloseFeatureDetails: () => void;
};

const TableViewer: React.FC<Props> = memo(
  ({
    fileContent,
    selectedFeature,
    onSingleClick,
    onDoubleClick,
    detectedGeometryType,
    totalFeatures,
    detailsOverlayOpen,
    detailsFeature,
    formattedData,
    onCloseFeatureDetails,
  }) => {
    const t = useT();

    // Handle row single click - select feature and show details
    const handleRowSingleClick = useCallback(
      (feature: any) => {
        onSingleClick?.(feature);
      },
      [onSingleClick],
    );

    // Handle row double click
    const handleRowDoubleClick = useCallback(
      (feature: any) => {
        onDoubleClick?.(feature);
      },
      [onDoubleClick],
    );

    // Loading state
    if (!fileContent || !formattedData.tableData) {
      return <BasicBoiler text={t("Loading data...")} className="h-full" />;
    }
    // No data state
    if (!formattedData.tableData || formattedData.tableData.length === 0) {
      return (
        <BasicBoiler
          text={t("No data to display in table format")}
          className="h-full"
        />
      );
    }

    return (
      <div className="relative flex h-full flex-col">
        <div className="flex h-full flex-1 flex-col">
          {/* Table */}
          <div className="flex-1 overflow-hidden">
            <VirtualizedTable
              columns={formattedData.tableColumns}
              data={formattedData.tableData}
              selectColumns={true}
              showFiltering={true}
              condensed={true}
              selectedRow={selectedFeature}
              useStrictSelectedRow={true}
              onRowClick={handleRowSingleClick}
              onRowDoubleClick={handleRowDoubleClick}
            />
          </div>

          {/* Status Display */}
          <div className="mt-1 flex items-center justify-between rounded-md px-2 text-xs text-muted-foreground">
            <div className="flex items-center gap-4">
              <span>
                {t("Rows")}:{" "}
                {(formattedData.tableData || []).length.toLocaleString()}
                {totalFeatures !== undefined &&
                  totalFeatures > 0 &&
                  ` / ${totalFeatures.toLocaleString()} ${t("total")}`}
              </span>
              {detectedGeometryType && (
                <span className="rounded px-2 text-xs">
                  {detectedGeometryType}
                </span>
              )}
              <span>
                {t("Columns")}: {(formattedData.tableColumns || []).length}
              </span>
            </div>
          </div>
        </div>

        {/* Feature Details Overlay */}

        {detailsOverlayOpen && (
          <FeatureDetailsOverlay
            feature={detailsFeature}
            detectedGeometryType={detectedGeometryType}
            onClose={onCloseFeatureDetails}
          />
        )}
      </div>
    );
  },
);

TableViewer.displayName = "TableViewer";

export default TableViewer;
