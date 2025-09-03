import {
  DotsThreeVerticalIcon,
  GlobeIcon,
  MapPinAreaIcon,
  TargetIcon,
  WarningIcon,
  DownloadIcon,
} from "@phosphor-icons/react";
import { memo, useCallback, useMemo, useState } from "react";

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
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
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
  
  // Streaming props
  isStreaming,
  streamingProgress,
  loadMore,
  detectedGeometryType,
}) => {
  const t = useT();
  const [tabValue, setTabValue] = useState<string>("2d-viewer");

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
    <Tabs
      className="h-full w-full"
      defaultValue={tabValue}
      onValueChange={setTabValue}>
      <div className="py-1">
        <TabsList className="flex w-full justify-between p-1">
          <div className="flex gap-2">
            <TabsTrigger
              className="gap-1 bg-card"
              value="2d-viewer"
              onClick={() => setTabValue("2d-viewer")}>
              <MapPinAreaIcon />
              <p className="text-sm font-thin select-none">{t("2D Viewer")}</p>
            </TabsTrigger>
            <TabsTrigger
              className="gap-1 bg-card"
              value="3d-viewer"
              onClick={() => setTabValue("3d-viewer")}>
              <GlobeIcon />
              <p className="text-sm font-thin select-none">{t("3D Viewer")}</p>
            </TabsTrigger>
          </div>
          
          {/* Streaming Progress Indicator */}
          {isStreaming && (
            <div className="flex items-center gap-2 text-xs">
              <div className="flex items-center gap-1 text-blue-600">
                <div className="h-2 w-2 rounded-full bg-blue-600 animate-pulse"></div>
                <span className="font-medium">STREAMING</span>
              </div>
              {streamingProgress && (
                <span className="text-muted-foreground">
                  {t("Features")}: {streamingProgress.featuresProcessed.toLocaleString()}
                  {streamingProgress.percentage && ` (${Math.round(streamingProgress.percentage)}%)`}
                </span>
              )}
              {detectedGeometryType && (
                <span className="rounded bg-muted px-2 py-1 text-xs">
                  {detectedGeometryType}
                </span>
              )}
            </div>
          )}
          {tabValue === "2d-viewer" && (
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
                {isStreaming && loadMore && (
                  <DropdownMenuItem onClick={loadMore}>
                    <DownloadIcon />
                    {t("Load More Data")}
                  </DropdownMenuItem>
                )}
              </DropdownMenuContent>
            </DropdownMenu>
          )}
        </TabsList>
      </div>
      {isLoadingData ? (
        <LoadingSkeleton />
      ) : (
        <>
          <TabsContent
            className="m-0 h-[calc(100%-32px)] overflow-hidden rounded-md"
            value="2d-viewer">
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
          </TabsContent>
          <TabsContent
            className="m-0 h-[calc(100%-32px)]"
            value="3d-viewer"
            id="cesiumContainer">
            <ThreeDViewer
              fileContent={selectedOutputData}
              fileType={fileType}
            />
          </TabsContent>
        </>
      )}
    </Tabs>
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
