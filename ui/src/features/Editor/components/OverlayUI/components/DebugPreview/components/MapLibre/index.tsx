import * as React from "react";
import { Map, Source } from "react-map-gl/maplibre";

import "maplibre-gl/dist/maplibre-gl.css";
import { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

type Props = {
  className?: string;
  fileContent: any | null;
  fileType: SupportedDataTypes | null;
};

const MapLibre: React.FC<Props> = ({ className, fileContent, fileType }) => {
  return (
    <div className={`relative size-full ${className}`}>
      <Map
        initialViewState={{
          latitude: 40,
          longitude: -100,
          zoom: 3,
        }}
        mapStyle="https://basemaps.cartocdn.com/gl/positron-gl-style/style.json"
        interactiveLayerIds={["data"]}>
        <Source type={fileType || "geojson"} data={fileContent} />
      </Map>
    </div>
  );
};

export { MapLibre };
