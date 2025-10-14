import maplibregl from "maplibre-gl";
import { useCallback, useMemo, useEffect, useState } from "react";
import { Map } from "react-map-gl/maplibre";

import { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

import "maplibre-gl/dist/maplibre-gl.css";
import Timeline from "./components/Timeline";
import { GeoJsonDataSource } from "./sources";
import {
  detectTemporalProperties,
  filterByTimelineValue,
} from "./utils/timelineUtils";

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
  // Timeline state
  const [selectedProperty, setSelectedProperty] = useState<string | null>(null);
  const [currentValue, setCurrentValue] = useState<string | number | null>(
    null,
  );

  // Detect temporal properties in GeoJSON
  const temporalProperties = useMemo(() => {
    if (fileType !== "geojson") return [];
    return detectTemporalProperties(fileContent);
  }, [fileType, fileContent]);

  // Auto-select first property when properties are detected
  useEffect(() => {
    if (temporalProperties.length > 0 && !selectedProperty) {
      const firstProp = temporalProperties[0];
      setSelectedProperty(firstProp.name);
      setCurrentValue(firstProp.max); // Start at most recent
    }
  }, [temporalProperties, selectedProperty]);

  // Filter data based on timeline
  const filteredContent = useMemo(() => {
    if (!selectedProperty || currentValue === null || fileType !== "geojson") {
      return fileContent;
    }
    return filterByTimelineValue(fileContent, selectedProperty, currentValue);
  }, [fileContent, fileType, selectedProperty, currentValue]);

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
            fileContent={filteredContent}
            enableClustering={enableClustering}
            selectedFeatureId={convertedSelectedFeature?.id}
          />
        )}
      </Map>

      {/* Timeline control - only show if temporal properties exist */}
      {temporalProperties.length > 0 && (
        <Timeline
          properties={temporalProperties}
          selectedProperty={selectedProperty}
          currentValue={currentValue}
          onPropertyChange={setSelectedProperty}
          onValueChange={setCurrentValue}
        />
      )}
    </div>
  );
};

export { MapLibre };
