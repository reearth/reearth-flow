import { Color, Viewer as CesiumViewerType } from "cesium";
import { useEffect, useRef, useState } from "react";
import { CesiumComponentRef, GeoJsonDataSource, Viewer } from "resium";

import { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

import { CesiumContents } from "./Contents";

const dummyCredit = document.createElement("div");

const defaultCesiumProps = {
  // timeline: false,
  // homeButton: false,
  // baseLayerPicker: false,
  // sceneModePicker: false,
  fullscreenButton: false,
  geocoder: false,
  animation: false,
  navigationHelpButton: false,
  creditContainer: dummyCredit,
};

type Props = {
  fileContent: string | null;
  fileType: SupportedDataTypes | null;
};

const CesiumViewer: React.FC<Props> = ({ fileContent, fileType }) => {
  const viewerRef = useRef<CesiumComponentRef<CesiumViewerType>>(null);
  const [isLoaded, setIsLoaded] = useState(false);

  useEffect(() => {
    if (isLoaded) return;
    setIsLoaded(true);
  }, [isLoaded]);

  return (
    <Viewer ref={viewerRef} full {...defaultCesiumProps}>
      <CesiumContents isLoaded={isLoaded} />
      {isLoaded && fileType === "geojson" && fileContent && (
        <GeoJsonDataSource
          data={fileContent}
          onLoad={(geoJsonDataSource) => {
            geoJsonDataSource.entities.values.forEach((entity) => {
              // TODO: Add more styling options
              if (entity.polygon) {
                entity.polygon.material = Color.BLACK.withAlpha(0.5);
                entity.polygon.outlineColor = Color.BLACK;
              }
            });

            if (viewerRef.current) {
              viewerRef.current.cesiumElement?.zoomTo(
                geoJsonDataSource.entities,
              );
            }
          }}
        />
      )}
    </Viewer>
  );
};
export { CesiumViewer };
