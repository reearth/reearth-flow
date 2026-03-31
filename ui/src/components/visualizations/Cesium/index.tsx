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

import useDoubleClick from "@flow/hooks/useDoubleClick";

import CityGmlData from "./CityGmlData";
import GeoJsonData from "./GeoJson";

const defaultCesiumProps: Partial<ViewerProps> = {
  // timeline: false,
  // baseLayerPicker: false,
  // sceneModePicker: false,
  fullscreenButton: false,
  sceneModePicker: false,
  infoBox: false,
  homeButton: false,
  geocoder: false,
  animation: false,
  requestRenderMode: true,
  maximumRenderTimeChange: Infinity,
  navigationHelpButton: false,
  creditContainer: document.createElement("none"),
};

type Props = {
  fileContent: any | null;
  visualizerType: "2d-map" | "3d-map";
  viewerRef?: React.RefObject<any>;
  selectedFeatureId?: string | null;
  detailsOverlayOpen: boolean;
  showSelectedFeatureOnly: boolean;
  onSelectedFeature?: (featureId: string | null) => void;
  onShowFeatureDetailsOverlay: (value: boolean) => void;
  setCityGmlBoundingSphere: (value: BoundingSphere | null) => void;
};

const CesiumViewer: React.FC<Props> = ({
  fileContent,
  visualizerType,
  viewerRef,
  selectedFeatureId,
  detailsOverlayOpen,
  showSelectedFeatureOnly,
  onSelectedFeature,
  onShowFeatureDetailsOverlay,
  setCityGmlBoundingSphere,
}) => {
  const [isLoaded, setIsLoaded] = useState(false);

  useEffect(() => {
    if (isLoaded) return;
    setIsLoaded(true);
  }, [isLoaded]);

  const pickOriginalId = useCallback(
    (movement: any) => {
      if (!viewerRef?.current?.cesiumElement) return null;
      const cesiumViewer = viewerRef.current.cesiumElement;
      const pickedObject = cesiumViewer.scene.pick(movement.position);
      if (!defined(pickedObject) || !defined(pickedObject.id)) return null;
      const pickedId = pickedObject.id;
      // Support both Entity (PropertyBag) and Primitive (plain object) ids
      return (
        pickedId?.properties?.getValue?.()?.["_originalId"] ??
        pickedId?._originalId ??
        null
      );
    },
    [viewerRef],
  );

  const onSingleClick = useCallback(
    (movement?: any) => {
      if (!onSelectedFeature || !movement) return;
      try {
        const originalId = pickOriginalId(movement);
        if (originalId) {
          onSelectedFeature(originalId);
        } else {
          onSelectedFeature(null);
          onShowFeatureDetailsOverlay(false);
        }
      } catch (e) {
        console.error("Cesium viewer error:", e);
      }
    },
    [onSelectedFeature, onShowFeatureDetailsOverlay, pickOriginalId],
  );

  const onDoubleClick = useCallback(
    (movement?: any) => {
      if (!onSelectedFeature || !movement) return;
      try {
        const originalId = pickOriginalId(movement);
        if (originalId) {
          onSelectedFeature(originalId);
          onShowFeatureDetailsOverlay(true);
        } else {
          onSelectedFeature(null);
          onShowFeatureDetailsOverlay(false);
        }
      } catch (e) {
        console.error("Cesium viewer error:", e);
      }
    },
    [onSelectedFeature, onShowFeatureDetailsOverlay, pickOriginalId],
  );

  const [handleSingleClick, handleDoubleClick] = useDoubleClick(
    onSingleClick,
    onDoubleClick,
  );

  // Separate features by geometry type
  const { geoJsonData, cityGmlData } = useMemo(() => {
    const features = fileContent?.features || [];

    const geoJsonFeatures: any[] = [];
    const cityGmlFeatures: any[] = [];

    for (const feature of features) {
      if (feature?.geometry?.type === "CityGmlGeometry") {
        cityGmlFeatures.push(feature);
      } else {
        geoJsonFeatures.push(feature);
      }
    }

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
    <Viewer
      ref={viewerRef}
      sceneMode={
        visualizerType === "2d-map" ? SceneMode.SCENE2D : SceneMode.SCENE3D
      }
      full
      {...defaultCesiumProps}>
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

      {isLoaded && (
        <>
          {/* Standard GeoJSON features */}
          {geoJsonData && (
            <GeoJsonData
              geoJsonData={geoJsonData}
              selectedFeatureId={selectedFeatureId}
              showSelectedFeatureOnly={showSelectedFeatureOnly}
            />
          )}

          {/* CityGML features */}
          {cityGmlData && (
            <CityGmlData
              cityGmlData={cityGmlData}
              setCityGmlBoundingSphere={setCityGmlBoundingSphere}
              selectedFeatureId={selectedFeatureId}
              detailsOverlayOpen={detailsOverlayOpen}
              showSelectedFeatureOnly={showSelectedFeatureOnly}
            />
          )}
        </>
      )}
    </Viewer>
  );
};
export { CesiumViewer };
