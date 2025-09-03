import { memo, useCallback, useEffect, useRef, useState } from "react";

import { RenderFallback, Button } from "@flow/components";
import { VirtualizedTable } from "@flow/components/visualizations/VirtualizedTable";
import useDataColumnizer from "@flow/hooks/useDataColumnizer";
import { useStreamingDataColumnizer } from "@flow/hooks/useStreamingDataColumnizer";
import { useT } from "@flow/lib/i18n";
import { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

import FeatureDetailsDialog from "./FeatureDetailsDialog";

type Props = {
  fileContent: any | null;
  fileType: SupportedDataTypes | null;
  selectedFeature: any;
  onSingleClick: (value: any) => void;
  onDoubleClick?: (value: any) => void;
  
  // Streaming props
  isStreaming?: boolean;
  streamingProgress?: {
    bytesProcessed: number;
    featuresProcessed: number;
    estimatedTotal?: number;
    percentage?: number;
  };
  loadMore?: () => void;
  detectedGeometryType?: string;
};

const TableViewer: React.FC<Props> = ({
  fileContent,
  fileType,
  selectedFeature,
  onSingleClick,
  onDoubleClick,
  
  // Streaming props
  isStreaming,
  streamingProgress,
  loadMore,
  detectedGeometryType,
}) => {
  const t = useT();
  
  // Use streaming columnizer for streaming data, traditional for small files
  const streamingColumnizer = useStreamingDataColumnizer({
    maxRows: 50000,
    autoExpandColumns: true,
  });
  
  const traditionalColumnizer = useDataColumnizer({
    parsedData: fileContent,
    type: fileType,
  });
  
  // Add new data to streaming columnizer when fileContent changes
  const prevFileContentLength = useRef(0);
  
  // Feature details dialog state
  const [selectedFeatureDetails, setSelectedFeatureDetails] = useState<any | null>(null);
  const [showDetailsDialog, setShowDetailsDialog] = useState(false);
  
  useEffect(() => {
    if (isStreaming && fileContent && Array.isArray(fileContent)) {
      // Only process if the array has actually grown
      if (fileContent.length > prevFileContentLength.current) {
        const newFeatures = fileContent.slice(prevFileContentLength.current);
        streamingColumnizer.addBatch(newFeatures);
        prevFileContentLength.current = fileContent.length;
      }
    } else if (!isStreaming) {
      // Reset when not streaming
      prevFileContentLength.current = 0;
    }
  }, [fileContent, isStreaming, streamingColumnizer]);
  
  // Handle feature details dialog
  const handleShowDetails = useCallback((feature: any) => {
    setSelectedFeatureDetails(feature);
    setShowDetailsDialog(true);
  }, []);
  
  const handleRowDoubleClick = useCallback((feature: any) => {
    handleShowDetails(feature);
    onDoubleClick?.(feature);
  }, [handleShowDetails, onDoubleClick]);
  
  // Choose which columnizer to use
  const tableData = isStreaming ? streamingColumnizer.tableData : traditionalColumnizer.tableData;
  const tableColumns = isStreaming ? streamingColumnizer.tableColumns : traditionalColumnizer.tableColumns;
  return (
    <RenderFallback
      message={t(
        "Table Viewer Could Not Be Loaded. Check if the data is valid.",
      )}
      textSize="sm">
      <div className="flex h-full flex-1 flex-col">
        {/* Streaming Status Header */}
        {isStreaming && (
          <div className="flex items-center justify-between border-b border-border bg-muted/50 px-3 py-2 text-xs text-muted-foreground">
            <div className="flex items-center gap-4">
              <span>
                {t("Rows")}: {(tableData || []).length.toLocaleString()}
                {streamingProgress && ` / ${streamingProgress.featuresProcessed.toLocaleString()} ${t("total")}`}
              </span>
              {detectedGeometryType && (
                <span className="rounded bg-muted px-2 py-1 text-xs">
                  {detectedGeometryType}
                </span>
              )}
              <span>
                {t("Columns")}: {(tableColumns || []).length}
                {isStreaming && streamingColumnizer.knownColumnCount > (tableColumns || []).length && 
                  ` (+${streamingColumnizer.knownColumnCount - (tableColumns || []).length} ${t("discovered")})`}
              </span>
            </div>
            {loadMore && (
              <Button variant="ghost" size="sm" onClick={loadMore}>
                {t("Load More")}
              </Button>
            )}
          </div>
        )}
        
        {/* Table */}
        <div className="flex-1 overflow-hidden">
          <VirtualizedTable
            columns={tableColumns}
            data={tableData}
            condensed
            selectColumns
            showFiltering
            selectedRow={selectedFeature}
            onRowClick={onSingleClick}
            onRowDoubleClick={handleRowDoubleClick}
            useStrictSelectedRow
          />
        </div>
      </div>
      
      {/* Feature Details Dialog */}
      <FeatureDetailsDialog
        feature={selectedFeatureDetails}
        open={showDetailsDialog}
        onOpenChange={setShowDetailsDialog}
      />
    </RenderFallback>
  );
};

export default memo(TableViewer);
