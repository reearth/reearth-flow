// import { Viewer as CesiumViewerType } from "cesium";
import { SceneMode } from "cesium";
import { useEffect, useState } from "react";
import { Viewer, ViewerProps } from "resium";

import { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

import CityGmlData from "./CityGmlData";
import GeoJsonData from "./GeoJson";

const defaultCesiumProps: Partial<ViewerProps> = {
  // timeline: false,
  // baseLayerPicker: false,
  // sceneModePicker: false,
  fullscreenButton: false,
  sceneModePicker: false,
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
};

const CesiumViewer: React.FC<Props> = ({ fileContent, fileType }) => {
  const [isLoaded, setIsLoaded] = useState(false);

  useEffect(() => {
    if (isLoaded) return;
    setIsLoaded(true);
  }, [isLoaded]);

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
    <Viewer full {...defaultCesiumProps}>
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
