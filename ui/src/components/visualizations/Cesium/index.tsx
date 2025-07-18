// import { Viewer as CesiumViewerType } from "cesium";
import { SceneMode } from "cesium";
import { useEffect, useState } from "react";
import { Viewer, ViewerProps } from "resium";

import { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

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

  return (
    <Viewer full {...defaultCesiumProps}>
      {isLoaded && fileType === "geojson" && (
        <GeoJsonData geoJsonData={fileContent} />
      )}
    </Viewer>
  );
};
export { CesiumViewer };
