import maplibregl from "maplibre-gl";
import * as React from "react";
import { useCallback } from "react";
import { Map } from "react-map-gl/maplibre";

import { SupportedDataTypes } from "@flow/hooks/useStreamingDebugRunQuery";

import "maplibre-gl/dist/maplibre-gl.css";
import { GeoJsonDataSource } from "./sources";

type Props = {
  className?: string;
  fileContent: any | null;
  fileType: SupportedDataTypes | null;
  enableClustering?: boolean;
  convertedSelectedFeature?: any;
  showSelectedFeatureOnly: boolean;
  mapRef: React.RefObject<maplibregl.Map | null>;
  onMapLoad: (onCenter?: boolean) => void;
  onSelectedFeature: (value: any) => void;
  onFlyToSelectedFeature?: (selectedFeature: any) => void;
  onShowFeatureDetailsOverlay: (value: boolean) => void;
};

const MapLibre: React.FC<Props> = ({
  className,
  fileContent,
  fileType,
  enableClustering,
  convertedSelectedFeature,
  showSelectedFeatureOnly,
  mapRef,
  onMapLoad,
  onSelectedFeature,
  onFlyToSelectedFeature,
  onShowFeatureDetailsOverlay,
}) => {
  const handleMapClick = useCallback(
    (e: maplibregl.MapLayerMouseEvent) => {
      if (e.features?.[0]?.id) {
        onSelectedFeature(e.features[0].id);
      } else {
        onSelectedFeature(null);
        onShowFeatureDetailsOverlay(false);
      }
    },
    [onSelectedFeature, onShowFeatureDetailsOverlay],
  );

  const handleMapDoubleClick = useCallback(
    (e: maplibregl.MapLayerMouseEvent) => {
      if (e.features?.[0]?.id) {
        onSelectedFeature(e.features[0].id);
        onFlyToSelectedFeature?.(e.features[0]);
        onShowFeatureDetailsOverlay(true);
      }
    },
    [onSelectedFeature, onFlyToSelectedFeature, onShowFeatureDetailsOverlay],
  );
  return (
    <div className={`relative size-full ${className}`}>
      <Map
        ref={(instance) => {
          if (instance) mapRef.current = instance.getMap();
        }}
        mapStyle="https://basemaps.cartocdn.com/gl/positron-gl-style/style.json"
        style={{ width: "100%", height: "100%" }}
        maplibreLogo={false}
        attributionControl={false}
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
            showSelectedFeatureOnly={showSelectedFeatureOnly}
          />
        )}
      </Map>
    </div>
  );
};

export { MapLibre };
