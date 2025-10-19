import {
  DotsThreeVerticalIcon,
  GlobeIcon,
  MapPinAreaIcon,
  TargetIcon,
} from "@phosphor-icons/react";
import { memo, useCallback, useMemo } from "react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
  LoadingSkeleton,
} from "@flow/components";
import ThreeJSViewer from "@flow/components/visualizations/ThreeJS";
import type { SupportedDataTypes } from "@flow/hooks/useStreamingDebugRunQuery";
import { useT } from "@flow/lib/i18n";
import type { JobState } from "@flow/stores";

import ThreeDViewer from "./components/ThreeDViewer";
import TwoDViewer from "./components/TwoDViewer";
import useHooks from "./hooks";

type Props = {
  fileType: SupportedDataTypes | null;
  selectedOutputData: any;
  debugJobState?: JobState;
  onConvertedSelectedFeature: (value: any) => void;
  dataURLs?: { key: string; name: string }[];
  selectedFeature: any;
  // enableClustering?: boolean;
  mapRef: React.RefObject<maplibregl.Map | null>;
  cesiumViewerRef: React.RefObject<any>;
  onSelectedFeature: (value: any) => void;
  // onEnableClusteringChange: (value: boolean) => void;
  onFlyToSelectedFeature?: (selectedFeature: any) => void;
  detectedGeometryType: string | null;
  visualizerType: "2d-map" | "3d-map" | "3d-model" | null;
  isComplete?: boolean;
};
const DebugPreview: React.FC<Props> = ({
  fileType,
  debugJobState,
  selectedOutputData,
  dataURLs,
  onConvertedSelectedFeature,
  mapRef,
  cesiumViewerRef,
  selectedFeature,
  onSelectedFeature,
  onFlyToSelectedFeature,
  detectedGeometryType,
  visualizerType,
  isComplete,
}) => {
  const t = useT();

  // Determine if we should show the viewer based on data availability
  const shouldShowViewer = () => {
    // Show viewer when:
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
        <div className="flex h-full items-center justify-center">
          <div className="text-center text-muted-foreground">
            <LoadingSkeleton className="mb-4" />
            <p className="text-sm">
              Loading data... {selectedOutputData?.features?.length || 0}{" "}
              features loaded
              {selectedOutputData?.features?.length >= 2000 && (
                <span className="mt-1 block text-xs">
                  (
                  {selectedOutputData?.features?.length >= 2000 && !isComplete
                    ? "Hit display limit of 2000 - viewer will show shortly"
                    : "Processing..."}
                  )
                </span>
              )}
            </p>
          </div>
        </div>
      ) : visualizerType === "2d-map" ? (
        <div className="h-full">
          {/* 2D Viewer Header with actions */}
          <div className="py-1">
            <div className="flex w-full justify-between p-1">
              <div className="flex items-center gap-1 px-2">
                <MapPinAreaIcon size={16} />
                <p className="text-sm font-medium select-none">
                  {t("2D Viewer")}
                </p>
                {detectedGeometryType && (
                  <span className="rounded px-2 text-xs text-muted-foreground">
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
                  {/* {fileType === "geojson" && (
                    <DropdownMenuCheckboxItem
                      checked={enableClustering}
                      onCheckedChange={(checked) =>
                        onEnableClusteringChange(!!checked)
                      }>
                      {t("Enable Clustering")}
                    </DropdownMenuCheckboxItem>
                  )} */}
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
              enableClustering={false}
              convertedSelectedFeature={convertedSelectedFeature}
              mapRef={mapRef}
              onMapLoad={handleMapLoad}
              onSelectedFeature={onSelectedFeature}
              onFlyToSelectedFeature={onFlyToSelectedFeature}
            />
          </div>
        </div>
      ) : visualizerType === "3d-map" ? (
        <div className="h-full">
          {/* 3D Viewer Header */}
          <div className="py-1">
            <div className="flex items-center gap-1 rounded-md px-3 py-2">
              <GlobeIcon size={16} />
              <p className="text-sm font-medium select-none">
                {t("3D Viewer")}
              </p>
              {detectedGeometryType && (
                <span className="rounded px-2 py-1 text-xs text-muted-foreground">
                  {detectedGeometryType}
                </span>
              )}
            </div>
          </div>
          <div className="h-[calc(100%-55px)]" id="cesiumContainer">
            <ThreeDViewer
              fileContent={selectedOutputData}
              fileType={fileType}
              cesiumViewerRef={cesiumViewerRef}
            />
          </div>
        </div>
      ) : visualizerType === "3d-model" ? (
        <div className="h-full">
          {/* 3D Model Viewer Header */}
          <div className="py-1">
            <div className="flex items-center gap-1 rounded-md px-3 py-2">
              <GlobeIcon size={16} />
              <p className="text-sm font-medium select-none">
                {t("3D Model Viewer")}
              </p>
              {detectedGeometryType && (
                <span className="rounded px-2 py-1 text-xs text-muted-foreground">
                  {detectedGeometryType}
                </span>
              )}
            </div>
          </div>
          <div className="h-[calc(100%-55px)]">
            <ThreeJSViewer fileContent={selectedOutputData} />
          </div>
        </div>
      ) : (
        <div className="flex h-full items-center justify-center text-muted-foreground">
          <div className="text-center">
            <p className="text-sm">
              {t("No viewer available for this data type")}
            </p>
            <p className="mt-1 text-xs">
              {t("Data type")}: {detectedGeometryType || "Unknown"}
            </p>
            <p className="mt-1 text-xs">
              {t("Visualizer")}: {visualizerType || "Unknown"}
            </p>
          </div>
        </div>
      )}
    </div>
  ) : null;
};

export default memo(DebugPreview);
