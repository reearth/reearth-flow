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
  onSelectedFeature?: (featureId: string | null) => void;
};

const CesiumViewer: React.FC<Props> = ({
  fileContent,
  fileType,
  viewerRef,
  onSelectedFeature,
}) => {
  const [isLoaded, setIsLoaded] = useState(false);
  const [cesiumReady, setCesiumReady] = useState(false);
  const [entitiesLoaded, setEntitiesLoaded] = useState(false);

  useEffect(() => {
    if (isLoaded) return;
    setIsLoaded(true);
  }, [isLoaded]);

  // Set up click handler when both Cesium viewer AND entities are ready
  useEffect(() => {
    if (!onSelectedFeature || !cesiumReady || !entitiesLoaded) return;

    const cesiumViewer = viewerRef?.current?.cesiumElement;
    if (!cesiumViewer) return;

    const handler = new ScreenSpaceEventHandler(cesiumViewer.scene.canvas);

    handler.setInputAction((movement: any) => {
      const pickedObject = cesiumViewer.scene.pick(movement.position);
      if (defined(pickedObject) && defined(pickedObject.id)) {
        const entity = pickedObject.id;
        if (entity.id) {
          try {
            onSelectedFeature(entity.id);
          } catch (e) {
            console.error("Cesium viewer error:", e);
          }
        }
      } else {
        onSelectedFeature(null);
      }
    }, ScreenSpaceEventType.LEFT_CLICK);

    return () => {
      handler.destroy();
    };
  }, [cesiumReady, entitiesLoaded, onSelectedFeature, viewerRef]);

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
    <Viewer
      ref={(viewer) => {
        if (viewerRef && viewer) {
          viewerRef.current = viewer;
          // Viewer is ready when ref callback fires
          setCesiumReady(true);
        }
      }}
      full
      {...defaultCesiumProps}>
      {isLoaded && fileType === "geojson" && (
        <>
          {/* Standard GeoJSON features */}
          {geoJsonFeatures.length > 0 && (
            <GeoJsonData
              geoJsonData={{
                type: "FeatureCollection",
                features: geoJsonFeatures,
              }}
              onEntitiesLoaded={() => setEntitiesLoaded(true)}
            />
          )}

          {/* CityGML features */}
          {cityGmlFeatures.length > 0 && (
            <CityGmlData
              cityGmlData={{
                type: "FeatureCollection",
                features: cityGmlFeatures,
              }}
              onEntitiesLoaded={() => setEntitiesLoaded(true)}
            />
          )}
        </>
      )}
    </Viewer>
  );
};
export { CesiumViewer };
