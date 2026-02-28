import {
  CornersOutIcon,
  DotsThreeVerticalIcon,
  EyeIcon,
  GlobeIcon,
  MapPinAreaIcon,
  TargetIcon,
} from "@phosphor-icons/react";
import { memo, useCallback, useMemo, useRef } from "react";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
} from "@flow/components";
import ThreeJSViewer, {
  type ThreeJSViewerRef,
} from "@flow/components/visualizations/ThreeJS";
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
  selectedFeatureId: string | null;
  // enableClustering?: boolean;
  mapRef: React.RefObject<maplibregl.Map | null>;
  cesiumViewerRef: React.RefObject<any>;
  onSelectedFeature: (value: any) => void;
  // onEnableClusteringChange: (value: boolean) => void;
  onFlyToSelectedFeature?: (selectedFeature: any) => void;
  onShowFeatureDetailsOverlay: (value: boolean) => void;
  detailsOverlayOpen: boolean;
  detectedGeometryType: string | null;
  visualizerType: "2d-map" | "3d-map" | "3d-model";
};
const DebugPreview: React.FC<Props> = ({
  fileType,
  debugJobState,
  selectedOutputData,
  dataURLs,
  onConvertedSelectedFeature,
  mapRef,
  cesiumViewerRef,
  selectedFeatureId,
  onSelectedFeature,
  onFlyToSelectedFeature,
  onShowFeatureDetailsOverlay,
  detailsOverlayOpen,
  detectedGeometryType,
  visualizerType,
}) => {
  const t = useT();
  const threeJSViewerRef = useRef<ThreeJSViewerRef>(null);

  const { featureMap, processedOutputData } = useMemo(() => {
    if (!selectedOutputData?.features) {
      return { featureMap: null, processedOutputData: selectedOutputData };
    }

    const map = new Map<string, any>();
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

  // Determine if we should show the viewer based on data availability
  const convertFeature = useCallback(
    (featureId: string | null) => {
      if (!featureId || !featureMap) return null;

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
    const converted = convertFeature(selectedFeatureId);
    onConvertedSelectedFeature(converted);
    return converted;
  }, [selectedFeatureId, onConvertedSelectedFeature, convertFeature]);

  const {
    showSelectedFeatureOnly,
    handleMapLoad,
    handleThreeDViewerReset,
    handleThreeJsReset,
    handleShowSelectedFeatureOnly,
    setCityGmlBoundingSphere,
  } = useHooks({
    mapRef,
    cesiumViewerRef,
    threeJSViewerRef,
    selectedOutputData,
    convertedSelectedFeature,
  });

  return debugJobState && dataURLs ? (
    <div className="h-full w-full">
      {visualizerType === "2d-map" ? (
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
                  <DropdownMenuItem onClick={() => handleMapLoad(true)}>
                    <CornersOutIcon />
                    {t("Center Data")}
                  </DropdownMenuItem>
                  <DropdownMenuItem
                    disabled={!convertedSelectedFeature}
                    onClick={() =>
                      onFlyToSelectedFeature?.(convertedSelectedFeature)
                    }>
                    <TargetIcon />
                    {t("Fly to Selected Feature")}
                  </DropdownMenuItem>
                  <DropdownMenuItem
                    disabled={!convertedSelectedFeature}
                    onClick={handleShowSelectedFeatureOnly}>
                    <EyeIcon />
                    {showSelectedFeatureOnly
                      ? t("Show All Features")
                      : t("Show Selected Feature Only")}
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
              showSelectedFeatureOnly={showSelectedFeatureOnly}
              onMapLoad={handleMapLoad}
              onSelectedFeature={onSelectedFeature}
              onFlyToSelectedFeature={onFlyToSelectedFeature}
              onShowFeatureDetailsOverlay={onShowFeatureDetailsOverlay}
            />
          </div>
        </div>
      ) : visualizerType === "3d-map" ? (
        <div className="h-full">
          {/* 3D Viewer Header */}
          <div className="py-1">
            <div className="flex w-full justify-between p-1">
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
                  <DropdownMenuItem onClick={handleThreeDViewerReset}>
                    <CornersOutIcon />
                    {t("Center Data")}
                  </DropdownMenuItem>
                  <DropdownMenuItem
                    disabled={!convertedSelectedFeature}
                    onClick={() =>
                      onFlyToSelectedFeature?.(convertedSelectedFeature)
                    }>
                    <TargetIcon />
                    {t("Fly to Selected Feature")}
                  </DropdownMenuItem>
                  <DropdownMenuItem
                    disabled={!convertedSelectedFeature}
                    onClick={handleShowSelectedFeatureOnly}>
                    <EyeIcon />
                    {showSelectedFeatureOnly
                      ? t("Show All Features")
                      : t("Show Selected Feature Only")}
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          </div>
          <div className="h-[calc(100%-55px)]" id="cesiumContainer">
            <ThreeDViewer
              fileContent={processedOutputData}
              fileType={fileType}
              cesiumViewerRef={cesiumViewerRef}
              selectedFeaturedId={selectedFeatureId}
              detailsOverlayOpen={detailsOverlayOpen}
              showSelectedFeatureOnly={showSelectedFeatureOnly}
              onSelectedFeature={onSelectedFeature}
              onShowFeatureDetailsOverlay={onShowFeatureDetailsOverlay}
              setCityGmlBoundingSphere={setCityGmlBoundingSphere}
            />
          </div>
        </div>
      ) : (
        <div className="h-full">
          {/* 3D Model Viewer Header with actions */}
          <div className="py-1">
            <div className="flex w-full justify-between p-1">
              <div className="flex items-center gap-1 px-2">
                <GlobeIcon size={16} />
                <p className="text-sm font-medium select-none">
                  {t("3D Model Viewer")}
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
                  <DropdownMenuItem onClick={handleThreeJsReset}>
                    <TargetIcon />
                    {t("Reset Camera")}
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          </div>
          <div className="h-[calc(100%-55px)]">
            <ThreeJSViewer
              ref={threeJSViewerRef}
              fileContent={selectedOutputData}
            />
          </div>
        </div>
      )}
    </div>
  ) : null;
};

export default memo(DebugPreview);
