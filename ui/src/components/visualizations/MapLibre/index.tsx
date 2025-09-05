import maplibregl from "maplibre-gl";
import * as React from "react";
import { useCallback } from "react";
import { Map } from "react-map-gl/maplibre";

import { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

import "maplibre-gl/dist/maplibre-gl.css";
import { GeoJsonDataSource } from "./sources";

type Props = {
  className?: string;
  fileContent: any | null;
  fileType: SupportedDataTypes | null;
  enableClustering?: boolean;
  convertedSelectedFeature?: any;
  mapRef: React.RefObject<maplibregl.Map | null>;
  onMapLoad: (onCenter?: boolean) => void;
  onSelectedFeature: (value: any) => void;
  onFlyToSelectedFeature?: (selectedFeature: any) => void;
};

const MapLibre: React.FC<Props> = ({
  className,
  fileContent,
  fileType,
  enableClustering,
  convertedSelectedFeature,
  mapRef,
  onMapLoad,
  onSelectedFeature,
  onFlyToSelectedFeature,
}) => {
  const handleMapClick = useCallback(
    (e: maplibregl.MapLayerMouseEvent) => {
      if (e.features?.[0]) {
        onSelectedFeature(e.features[0]);
      } else {
        onSelectedFeature(undefined);
      }
    },
    [onSelectedFeature],
  );

  const handleMapDoubleClick = useCallback(
    (e: maplibregl.MapLayerMouseEvent) => {
      if (e.features?.[0]) {
        onSelectedFeature(e.features[0]);
        onFlyToSelectedFeature?.(e.features[0]);
      }
    },
    [onSelectedFeature, onFlyToSelectedFeature],
  );
  return (
    <div className={`relative size-full ${className}`}>
      <Map
        ref={(instance) => {
          if (instance) mapRef.current = instance.getMap();
        }}
        mapStyle="https://basemaps.cartocdn.com/gl/positron-gl-style/style.json"
        style={{ width: "100%", height: "100%" }}
        maplibreLogo={true}
        interactiveLayerIds={["point-layer", "line-layer", "polygon-layer"]}
        onClick={handleMapClick}
        onDblClick={handleMapDoubleClick}
        onLoad={() => onMapLoad()}>
        {fileType === "geojson" && (
          <GeoJsonDataSource
            key={`geojson-source-${enableClustering}`}
            mapRef={mapRef}
            fileType={fileType}
            fileContent={fileContent}
            enableClustering={enableClustering}
            selectedFeatureId={convertedSelectedFeature?.id}
          />
        )}
      </Map>
    </div>
  );
};

export { MapLibre };
