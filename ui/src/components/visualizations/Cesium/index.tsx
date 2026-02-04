// import { Viewer as CesiumViewerType } from "cesium";
import { defined, SceneMode, ScreenSpaceEventType } from "cesium";
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
  onSelectedFeature?: (featureId: string | null) => void;
};

const CesiumViewer: React.FC<Props> = ({
  fileContent,
  fileType,
  viewerRef,
  onSelectedFeature,
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
        const entity = pickedObject.id;
        const properties = entity.properties?.getValue?.();
        if (properties?._originalId) {
          try {
            onSelectedFeature(properties?._originalId);
          } catch (e) {
            console.error("Cesium viewer error:", e);
          }
        }
      } else {
        onSelectedFeature(null);
      }
    },
    [onSelectedFeature, viewerRef],
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
        </ScreenSpaceEventHandler>
      )}

      {isLoaded && fileType === "geojson" && (
        <>
          {/* Standard GeoJSON features */}
          {geoJsonData && <GeoJsonData geoJsonData={geoJsonData} />}

          {/* CityGML features */}
          {cityGmlData && <CityGmlData cityGmlData={cityGmlData} />}
        </>
      )}
    </Viewer>
  );
};
export { CesiumViewer };
