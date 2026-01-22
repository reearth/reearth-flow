// import { Viewer as CesiumViewerType } from "cesium";
import { defined, SceneMode, ScreenSpaceEventType } from "cesium";
import { useCallback, useEffect, useState } from "react";
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
  entityMapRef?: React.RefObject<Map<string | number, any>>;
};

const CesiumViewer: React.FC<Props> = ({
  fileContent,
  fileType,
  viewerRef,
  onSelectedFeature,
  entityMapRef,
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

        if (entity.id) {
          try {
            // Check for compound IDs (CityGML surfaces like "buildingId_wall_1")
            if (entity.id.includes("_")) {
              onSelectedFeature(entity.id.split("_")[0]);
            } else {
              onSelectedFeature(entity.id);
            }
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
          {geoJsonFeatures.length > 0 && (
            <GeoJsonData
              geoJsonData={{
                type: "FeatureCollection",
                features: geoJsonFeatures,
              }}
              entityMapRef={entityMapRef}
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
