import {
  DotsThreeVerticalIcon,
  GlobeIcon,
  MapPinAreaIcon,
  TargetIcon,
  WarningIcon,
} from "@phosphor-icons/react";
import { memo, useCallback, useMemo } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
  LoadingSkeleton,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { JobState } from "@flow/stores";
import type { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

import ThreeDViewer from "./components/ThreeDViewer";
import TwoDViewer from "./components/TwoDViewer";
import useHooks from "./hooks";

type Props = {
  fileType: SupportedDataTypes | null;
  selectedOutputData: any;
  debugJobState?: JobState;
  isLoadingData: boolean;
  onConvertedSelectedFeature: (value: any) => void;
  dataURLs?: { key: string; name: string }[];
  showTempPossibleIssuesDialog: boolean;
  selectedFeature: any;
  enableClustering?: boolean;
  mapRef: React.RefObject<maplibregl.Map | null>;
  onShowTempPossibleIssuesDialogClose: () => void;
  onSelectedFeature: (value: any) => void;
  onEnableClusteringChange: (value: boolean) => void;
  onFlyToSelectedFeature?: (selectedFeature: any) => void;
  detectedGeometryType: string | null;
  isStreaming?: boolean;
  isComplete?: boolean;
};
const DebugPreview: React.FC<Props> = ({
  fileType,
  debugJobState,
  selectedOutputData,
  dataURLs,
  onConvertedSelectedFeature,
  isLoadingData,
  showTempPossibleIssuesDialog,
  enableClustering,
  mapRef,
  selectedFeature,
  onShowTempPossibleIssuesDialogClose,
  onSelectedFeature,
  onEnableClusteringChange,
  onFlyToSelectedFeature,
  detectedGeometryType,
  isStreaming,
  isComplete,
}) => {
  const t = useT();
  
  // Auto-detect which viewer to show based on geometry type
  const getViewerType = (detectedType: string | null): "2d" | "3d" | null => {
    if (!detectedType) return null; // Unknown type - no viewer
    
    switch (detectedType) {
      case "CityGmlGeometry":
      case "FlowGeometry3D":
        return "3d";
      case "FlowGeometry2D":
        return "2d";
      default:
        return null; // Unknown type - no viewer
    }
  };
  
  const viewerType = getViewerType(detectedGeometryType);
  
  // Determine if we should show the viewer based on streaming state
  const shouldShowViewer = () => {
    // For non-streaming data, use the existing loading logic
    if (!isStreaming) {
      return !isLoadingData;
    }
    
    // For streaming data, show viewer when:
    // 1. Stream is complete (we have the total count)
    // 2. OR we have data and hit the display limit (2000 features)
    const hasData = selectedOutputData?.features?.length > 0;
    const hitDisplayLimit = selectedOutputData?.features?.length >= 2000;
    
    return hasData && (isComplete || hitDisplayLimit);
  };

  const { handleMapLoad } = useHooks({
    mapRef,
    selectedOutputData,
  });

  const { featureMap, processedOutputData } = useMemo(() => {
    if (!selectedOutputData?.features) {
      return { featureMap: null, processedOutputData: selectedOutputData };
    }

    const map = new Map<string | number, any>();
    const processedFeatures = selectedOutputData.features.map((f: any) => {
      const processedFeature = {
        ...f,
        properties: {
          _originalId: f.id,
          ...f.properties,
        },
      };

      if (f.id !== undefined) {
        map.set(f.id, processedFeature);
      }

      return processedFeature;
    });

    return {
      featureMap: map,
      processedOutputData: {
        ...selectedOutputData,
        features: processedFeatures,
      },
    };
  }, [selectedOutputData]);

  const convertFeature = useCallback(
    (feature: any) => {
      if (!feature || !featureMap) return null;

      if ("geometry" in feature && feature.geometry) {
        return feature;
      }

      const featureId = feature.properties?._originalId ?? feature.id;
      if (featureId === undefined) return null;

      let normalizedId = featureId;
      if (typeof featureId === "string") {
        try {
          normalizedId = JSON.parse(featureId);
        } catch {
          normalizedId = featureId;
        }
      }

      return featureMap.get(normalizedId) || null;
    },
    [featureMap],
  );

  const convertedSelectedFeature = useMemo(() => {
    const converted = convertFeature(selectedFeature);
    onConvertedSelectedFeature(converted);
    return converted;
  }, [selectedFeature, onConvertedSelectedFeature, convertFeature]);

  return debugJobState && dataURLs ? (
    <div className="h-full w-full">
      {!shouldShowViewer() ? (
        <div className="h-full flex items-center justify-center">
          <div className="text-center text-muted-foreground">
            <LoadingSkeleton className="mb-4" />
            <p className="text-sm">
              {isStreaming ? (
                <>
                  Loading streaming data... {selectedOutputData?.features?.length || 0} features loaded
                  {selectedOutputData?.features?.length >= 2000 && 
                    <span className="block text-xs mt-1">
                      ({(selectedOutputData?.features?.length >= 2000 && !isComplete) ? 
                        'Hit display limit of 2000 - viewer will show shortly' : 
                        'Processing...'}
                      )
                    </span>
                  }
                </>
              ) : (
                'Loading data...'
              )}
            </p>
          </div>
        </div>
      ) : viewerType === "2d" ? (
        <div className="h-full">
          {/* 2D Viewer Header with actions */}
          <div className="py-1">
            <div className="flex w-full justify-between rounded-md bg-muted/30 p-1">
              <div className="flex items-center gap-1 px-2">
                <MapPinAreaIcon size={16} />
                <p className="text-sm font-medium select-none">{t("2D Viewer")}</p>
                {detectedGeometryType && (
                  <span className="rounded bg-muted px-2 py-1 text-xs text-muted-foreground">
                    {detectedGeometryType}
                  </span>
                )}
              </div>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <IconButton
                    className="w-[25px]"
                    tooltipText={t("Additional actions")}
                    tooltipOffset={12}
                    icon={<DotsThreeVerticalIcon size={18} />}
                  />
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  <DropdownMenuCheckboxItem
                    checked={enableClustering}
                    onCheckedChange={(checked) =>
                      onEnableClusteringChange(!!checked)
                    }>
                    {t("Enable Clustering")}
                  </DropdownMenuCheckboxItem>
                  <DropdownMenuItem onClick={() => handleMapLoad(true)}>
                    <TargetIcon />
                    {t("Center Data")}
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          </div>
          <div className="h-[calc(100%-55px)] overflow-hidden rounded-md">
            <TwoDViewer
              fileContent={processedOutputData}
              fileType={fileType}
              enableClustering={enableClustering}
              convertedSelectedFeature={convertedSelectedFeature}
              mapRef={mapRef}
              onMapLoad={handleMapLoad}
              onSelectedFeature={onSelectedFeature}
              onFlyToSelectedFeature={onFlyToSelectedFeature}
            />
          </div>
        </div>
      ) : viewerType === "3d" ? (
        <div className="h-full">
          {/* 3D Viewer Header */}
          <div className="py-1">
            <div className="flex items-center gap-1 rounded-md bg-muted/30 px-3 py-2">
              <GlobeIcon size={16} />
              <p className="text-sm font-medium select-none">{t("3D Viewer")}</p>
              {detectedGeometryType && (
                <span className="rounded bg-muted px-2 py-1 text-xs text-muted-foreground">
                  {detectedGeometryType}
                </span>
              )}
            </div>
          </div>
          <div className="h-[calc(100%-55px)]" id="cesiumContainer">
            <ThreeDViewer
              fileContent={selectedOutputData}
              fileType={fileType}
            />
          </div>
        </div>
      ) : (
        <div className="flex h-full items-center justify-center text-muted-foreground">
          <div className="text-center">
            <p className="text-sm">{t("No viewer available for this data type")}</p>
            <p className="mt-1 text-xs">{t("Data type")}: {detectedGeometryType || "Unknown"}</p>
          </div>
        </div>
      )}
    </div>
  ) : showTempPossibleIssuesDialog ? (
    <Dialog open={showTempPossibleIssuesDialog}>
      <DialogContent size="sm" hideCloseButton>
        <DialogHeader className="text-warning">
          <DialogTitle className="flex justify-center gap-1">
            <WarningIcon weight="light" />
            {t("Warning")}
          </DialogTitle>
        </DialogHeader>
        <DialogContentWrapper>
          <DialogContentSection>
            <p className="text-sm font-light">
              {t("Your workflow completed without any output data.")}
            </p>
          </DialogContentSection>
          <DialogContentSection>
            <p className="text-sm font-light">
              {t(
                "Please review the logs to see if there were any errors during the workflow process.",
              )}
            </p>
          </DialogContentSection>
        </DialogContentWrapper>
        <DialogFooter>
          <Button
            className="self-end"
            size="sm"
            onClick={onShowTempPossibleIssuesDialogClose}>
            {t("OK")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  ) : null;
};

export default memo(DebugPreview);
