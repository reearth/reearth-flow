import { memo, useCallback, useEffect, useMemo, useRef, useState } from "react";

import BasicBoiler from "@flow/components/BasicBoiler";
import { VirtualizedTable } from "@flow/components/visualizations/VirtualizedTable";
import useDataColumnizer from "@flow/hooks/useDataColumnizer";
import { SupportedDataTypes } from "@flow/hooks/useStreamingDebugRunQuery";
import { useT } from "@flow/lib/i18n";

import FeatureDetailsOverlay from "./FeatureDetailsOverlay";

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
    const [detailsOverlayOpen, setDetailsOverlayOpen] = useState(false);
    const [detailsFeature, setDetailsFeature] = useState<any>(null);
    const previousSelectedFeature = useRef<any>(null);

    // Use traditional columnizer for all data (streaming is now pre-transformed)
    const columnizer = useDataColumnizer({
      parsedData: fileContent,
      type: fileType,
    });

    // Create a Map for O(1) feature lookup by ID
    const featureIdMap = useMemo(() => {
      if (!columnizer.tableData) return null;

      const map = new Map<string | number, any>();
      columnizer.tableData.forEach((row: any) => {
        const id = row.id ?? row.properties?._originalId ?? row.properties?.id;
        if (id !== null && id !== undefined) {
          map.set(id, row);
        }
      });
      return map;
    }, [columnizer.tableData]);

    // Handle showing feature details
    const handleShowFeatureDetails = useCallback((feature: any) => {
      setDetailsFeature(feature);
      setDetailsOverlayOpen(true);
    }, []);

    // Handle closing feature details
    const handleCloseFeatureDetails = useCallback(() => {
      setDetailsOverlayOpen(false);
      setDetailsFeature(null);
    }, []);

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
        handleShowFeatureDetails(feature);
      },
      [onDoubleClick, handleShowFeatureDetails],
    );

    useEffect(() => {
      if (!selectedFeature || !detailsOverlayOpen || !featureIdMap) {
        return;
      }
      const prevId = previousSelectedFeature.current?.id;
      const currId = selectedFeature.id;

      if (prevId !== currId) {
        const matchingRow =
          featureIdMap.get(JSON.stringify(currId)) ?? selectedFeature;
        previousSelectedFeature.current = selectedFeature;
        handleShowFeatureDetails(matchingRow);
      }
    }, [
      handleShowFeatureDetails,
      selectedFeature,
      detailsOverlayOpen,
      featureIdMap,
    ]);

    // Loading state
    if (!fileContent || !columnizer.tableData) {
      return <BasicBoiler text={t("Loading data...")} className="h-full" />;
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
      <div className="relative flex h-full flex-col">
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
              onRowClick={handleRowSingleClick}
              onRowDoubleClick={handleRowDoubleClick}
            />
          </div>

          {/* Status Display */}
          <div className="mt-1 flex items-center justify-between rounded-md px-2 text-xs text-muted-foreground">
            <div className="flex items-center gap-4">
              <span>
                {t("Rows")}:{" "}
                {(columnizer.tableData || []).length.toLocaleString()}
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
                {t("Columns")}: {(columnizer.tableColumns || []).length}
              </span>
            </div>
          </div>
        </div>

        {/* Feature Details Overlay */}
        <FeatureDetailsOverlay
          feature={detailsFeature}
          isOpen={detailsOverlayOpen}
          onClose={handleCloseFeatureDetails}
          detectedGeometryType={detectedGeometryType}
        />
      </div>
    );
  },
);

TableViewer.displayName = "TableViewer";

export default TableViewer;
