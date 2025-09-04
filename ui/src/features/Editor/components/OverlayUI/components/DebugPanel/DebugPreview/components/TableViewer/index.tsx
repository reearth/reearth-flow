import { memo, useCallback, useEffect, useRef, useState } from "react";

import { RenderFallback } from "@flow/components";
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
  detectedGeometryType?: string;
  totalFeatures?: number;
};

const TableViewer: React.FC<Props> = ({
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
  const prevFileContentRef = useRef<any[]>([]);
  
  // Feature details dialog state
  const [selectedFeatureDetails, setSelectedFeatureDetails] = useState<any | null>(null);
  const [showDetailsDialog, setShowDetailsDialog] = useState(false);
  
  useEffect(() => {
    if (fileContent && Array.isArray(fileContent) && fileContent.length > 0) {
      // Check if this is a new file by comparing actual data content, not just length
      const isNewFile = prevFileContentLength.current === 0 || 
                       prevFileContentLength.current > fileContent.length ||
                       (fileContent.length > 0 && prevFileContentRef.current.length > 0 && 
                        JSON.stringify(fileContent[0]) !== JSON.stringify(prevFileContentRef.current[0]));
      
      if (isNewFile) {
        // Reset columnizer for new file (both streaming and cached data)
        streamingColumnizer.reset();
        streamingColumnizer.addBatch(fileContent);
        prevFileContentLength.current = fileContent.length;
        prevFileContentRef.current = fileContent;
      } else if (isStreaming && fileContent.length > prevFileContentLength.current) {
        // Incremental update for same file during streaming
        const newFeatures = fileContent.slice(prevFileContentLength.current);
        streamingColumnizer.addBatch(newFeatures);
        prevFileContentLength.current = fileContent.length;
        prevFileContentRef.current = fileContent;
      }
    } else if ((!fileContent || fileContent.length === 0) && prevFileContentLength.current > 0) {
      // File was cleared or switched to empty - reset
      streamingColumnizer.reset();
      prevFileContentLength.current = 0;
      prevFileContentRef.current = [];
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
            {/* Streaming Status Header */}
        {isStreaming && (
          <div className="mt-1 flex items-center justify-between rounded-md bg-muted/50 px-3 py-1 text-xs text-muted-foreground">
            <div className="flex items-center gap-4">
              <span>
                {t("Rows")}: {(tableData || []).length.toLocaleString()}
                {isStreaming && totalFeatures !== undefined && 
                  ` / ${totalFeatures.toLocaleString()} ${t("total")}`}
              </span>
              {detectedGeometryType && (
                <span className="rounded bg-muted px-2 text-xs">
                  {detectedGeometryType}
                </span>
              )}
              <span>
                {t("Columns")}: {(tableColumns || []).length}
                {isStreaming && streamingColumnizer.knownColumnCount > (tableColumns || []).length && 
                  ` (+${streamingColumnizer.knownColumnCount - (tableColumns || []).length} ${t("discovered")})`}
              </span>
            </div>
          </div>
        )}
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
