// import { Viewer as CesiumViewerType } from "cesium";
import {
  BoundingSphere,
  defined,
  SceneMode,
  ScreenSpaceEventType,
} from "cesium";
import { useCallback, useEffect, useMemo, useState } from "react";
import {
  ScreenSpaceEvent,
  ScreenSpaceEventHandler,
  Viewer,
  ViewerProps,
} from "resium";

import { SupportedDataTypes } from "@flow/hooks/useStreamingDebugRunQuery";

import CityGmlData from "./CityGmlData";
import GeoJsonData from "./GeoJson";

const defaultCesiumProps: Partial<ViewerProps> = {
  // timeline: false,
  // baseLayerPicker: false,
  // sceneModePicker: false,
  fullscreenButton: false,
  sceneModePicker: false,
  infoBox: false,
  sceneMode: SceneMode.SCENE3D,
  homeButton: false,
  geocoder: false,
  animation: false,
  navigationHelpButton: false,
  creditContainer: document.createElement("none"),
};

type Props = {
  fileContent: any | null;
  fileType: SupportedDataTypes | null;
  viewerRef?: React.RefObject<any>;
  selectedFeatureId?: string | null;
  detailsOverlayOpen: boolean;
  onSelectedFeature?: (featureId: string | null) => void;
  onShowFeatureDetailsOverlay: (value: boolean) => void;
  setCityGmlBoundingSphere: (value: BoundingSphere | null) => void;
};

const CesiumViewer: React.FC<Props> = ({
  fileContent,
  fileType,
  viewerRef,
  selectedFeatureId,
  detailsOverlayOpen,
  onSelectedFeature,
  onShowFeatureDetailsOverlay,
  setCityGmlBoundingSphere,
}) => {
  const [isLoaded, setIsLoaded] = useState(false);

  useEffect(() => {
    if (isLoaded) return;
    setIsLoaded(true);
  }, [isLoaded]);

  const handleSingleClick = useCallback(
    (movement: any) => {
      if (!onSelectedFeature || !viewerRef?.current?.cesiumElement) return;

      const cesiumViewer = viewerRef.current.cesiumElement;
      const pickedObject = cesiumViewer.scene.pick(movement.position);

      if (defined(pickedObject) && defined(pickedObject.id)) {
        const pickedId = pickedObject.id;
        // Support both Entity (PropertyBag) and Primitive (plain object) ids
        const originalId =
          pickedId?.properties?.getValue?.()?.["_originalId"] ??
          pickedId?._originalId;
        if (originalId) {
          try {
            onSelectedFeature(originalId);
          } catch (e) {
            console.error("Cesium viewer error:", e);
          }
        }
      } else {
        onSelectedFeature(null);
        onShowFeatureDetailsOverlay(false);
      }
    },
    [onSelectedFeature, onShowFeatureDetailsOverlay, viewerRef],
  );

  const handleDoubleClick = useCallback(
    (movement: any) => {
      if (!onSelectedFeature || !viewerRef?.current?.cesiumElement) return;

      const cesiumViewer = viewerRef.current.cesiumElement;
      const pickedObject = cesiumViewer.scene.pick(movement.position);

      if (defined(pickedObject) && defined(pickedObject.id)) {
        const pickedId = pickedObject.id;
        // Support both Entity (PropertyBag) and Primitive (plain object) ids
        const originalId =
          pickedId?.properties?.getValue?.()?.["_originalId"] ??
          pickedId?._originalId;
        if (originalId) {
          try {
            onSelectedFeature(originalId);
            onShowFeatureDetailsOverlay(true);
          } catch (e) {
            console.error("Cesium viewer error:", e);
          }
        }
      } else {
        onSelectedFeature(null);
        onShowFeatureDetailsOverlay(false);
      }
    },
    [onSelectedFeature, onShowFeatureDetailsOverlay, viewerRef],
  );

  // Separate features by geometry type
  const { geoJsonData, cityGmlData } = useMemo(() => {
    const features = fileContent?.features || [];

    const geoJsonFeatures = features.filter(
      (feature: any) => feature?.geometry?.type !== "CityGmlGeometry",
    );

    const cityGmlFeatures = features.filter(
      (feature: any) => feature?.geometry?.type === "CityGmlGeometry",
    );

    return {
      geoJsonData:
        geoJsonFeatures.length > 0
          ? { type: "FeatureCollection" as const, features: geoJsonFeatures }
          : null,
      cityGmlData:
        cityGmlFeatures.length > 0
          ? { type: "FeatureCollection" as const, features: cityGmlFeatures }
          : null,
    };
  }, [fileContent]);

  return (
    <Viewer ref={viewerRef} full {...defaultCesiumProps}>
      {onSelectedFeature && (
        <ScreenSpaceEventHandler>
          <ScreenSpaceEvent
            action={handleSingleClick}
            type={ScreenSpaceEventType.LEFT_CLICK}
          />
          <ScreenSpaceEvent
            action={handleDoubleClick}
            type={ScreenSpaceEventType.LEFT_DOUBLE_CLICK}
          />
        </ScreenSpaceEventHandler>
      )}

      {isLoaded && fileType === "geojson" && (
        <>
          {/* Standard GeoJSON features */}
          {geoJsonData && (
            <GeoJsonData
              geoJsonData={geoJsonData}
              selectedFeatureId={selectedFeatureId}
            />
          )}

          {/* CityGML features */}
          {cityGmlData && (
            <CityGmlData
              cityGmlData={cityGmlData}
              setCityGmlBoundingSphere={setCityGmlBoundingSphere}
              selectedFeatureId={selectedFeatureId}
              detailsOverlayOpen={detailsOverlayOpen}
            />
          )}
        </>
      )}
    </Viewer>
  );
};
export { CesiumViewer };
