// import { Viewer as CesiumViewerType } from "cesium";
import {
  defined,
  SceneMode,
  ScreenSpaceEventHandler,
  ScreenSpaceEventType,
} from "cesium";
import { useEffect, useState } from "react";
import { Viewer, ViewerProps } from "resium";

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
  onSelectedFeature?: (featureId: any) => void;
};

const CesiumViewer: React.FC<Props> = ({
  fileContent,
  fileType,
  viewerRef,
  onSelectedFeature,
}) => {
  const [isLoaded, setIsLoaded] = useState(false);
  const [cesiumReady, setCesiumReady] = useState(false);

  // Check if Cesium viewer is ready
  useEffect(() => {
    if (cesiumReady) return;

    const checkInterval = setInterval(() => {
      if (viewerRef?.current?.cesiumElement) {
        setCesiumReady(true);
        clearInterval(checkInterval);
      }
    }, 100);

    return () => clearInterval(checkInterval);
  }, [viewerRef, cesiumReady]);

  useEffect(() => {
    if (isLoaded) return;
    setIsLoaded(true);
  }, [isLoaded]);

  useEffect(() => {
    if (!onSelectedFeature || !cesiumReady) return;

    const cesiumViewer = viewerRef?.current?.cesiumElement;
    if (!cesiumViewer) return;

    const handler = new ScreenSpaceEventHandler(cesiumViewer.scene.canvas);

    handler.setInputAction((movement: any) => {
      const pickedObject = cesiumViewer.scene.pick(movement.position);
      if (defined(pickedObject) && defined(pickedObject.id)) {
        const entity = pickedObject.id;
        if (entity._id) {
          try {
            onSelectedFeature(entity._id);
          } catch (e) {
            console.error("Cesium viewer error:", e);
          }
        }
      } else {
        onSelectedFeature(undefined);
      }
    }, ScreenSpaceEventType.LEFT_CLICK);

    return () => {
      handler.destroy();
    };
  }, [viewerRef, cesiumReady, onSelectedFeature]);

  // Separate features by geometry type
  const geoJsonFeatures =
    fileContent?.features?.filter(
      (feature: any) => feature?.geometry?.type !== "CityGmlGeometry",
    ) || [];

  const cityGmlFeatures =
    fileContent?.features?.filter(
      (feature: any) => feature?.geometry?.type === "CityGmlGeometry",
    ) || [];

  return (
    <Viewer ref={viewerRef} full {...defaultCesiumProps}>
      {isLoaded && fileType === "geojson" && (
        <>
          {/* Standard GeoJSON features */}
          {geoJsonFeatures.length > 0 && (
            <GeoJsonData
              geoJsonData={{
                type: "FeatureCollection",
                features: geoJsonFeatures,
              }}
            />
          )}

          {/* CityGML features */}
          {cityGmlFeatures.length > 0 && (
            <CityGmlData
              cityGmlData={{
                type: "FeatureCollection",
                features: cityGmlFeatures,
              }}
            />
          )}
        </>
      )}
    </Viewer>
  );
};
export { CesiumViewer };
