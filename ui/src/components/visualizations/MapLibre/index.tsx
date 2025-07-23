import { MapPinAreaIcon } from "@phosphor-icons/react";
import { Cross2Icon } from "@radix-ui/react-icons";
import bbox from "@turf/bbox";
import maplibregl, { LngLatBounds } from "maplibre-gl";
import * as React from "react";
import { useRef, useEffect, useState, useMemo, useCallback } from "react";
import { Map } from "react-map-gl/maplibre";

import { Button, IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

import "maplibre-gl/dist/maplibre-gl.css";
import { GeoJsonDataSource } from "./sources";

type Props = {
  className?: string;
  fileContent: any | null;
  fileType: SupportedDataTypes | null;
  enableClustering?: boolean;
  selectedFeature?: any;
  onSelectedFeature: (value: any) => void;
};

type MapSidePanelProps = {
  mapFeature: any;
  setMapFeature: (value: any) => void;
  onFlyToSelectedFeature?: (selectedFeature: any) => void;
};

const MapLibre: React.FC<Props> = ({
  className,
  fileContent,
  fileType,
  enableClustering,
  onSelectedFeature,
  selectedFeature,
}) => {
  const mapRef = useRef<maplibregl.Map | null>(null);
  const [mapFeature, setMapFeature] = useState<any>(null);
  console.log("MAP LIBRE", selectedFeature, mapFeature);

  const dataBounds = useMemo(() => {
    if (!fileContent) return null;

    try {
      const [minLng, minLat, maxLng, maxLat] = bbox(fileContent);
      return new LngLatBounds([minLng, minLat], [maxLng, maxLat]);
    } catch (err) {
      console.error("Error computing bbox:", err);
      return null;
    }
  }, [fileContent]);

  const handleMapLoad = useCallback(() => {
    if (mapRef.current && dataBounds) {
      mapRef.current.fitBounds(dataBounds, {
        padding: 40,
        duration: 0,
        maxZoom: 16,
      });
    }
  }, [dataBounds]);

  const handleFlyToSelectedFeature = useCallback(() => {
    if (mapRef.current && selectedFeature) {
      try {
        const [minLng, minLat, maxLng, maxLat] = bbox(selectedFeature);
        mapRef.current.fitBounds(
          [
            [minLng, minLat],
            [maxLng, maxLat],
          ],

          { padding: 40, duration: 500, maxZoom: 12 },
        );
      } catch (err) {
        console.error("Error computing bbox for selectedFeature:", err);
      }
    }
  }, [selectedFeature]);
  const normalizedFileContent = useMemo(() => {
    if (!fileContent) return null;

    return {
      ...fileContent,
      features: fileContent.features.map((f: any) => ({
        ...f,
        properties: {
          ...f.properties,
          id: f.id,
        },
      })),
    };
  }, [fileContent]);
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
        onClick={(e) => {
          const feature = e.features?.find((f) => f.layer.id === "point-layer");
          if (feature) {
            setMapFeature(feature);
          }
        }}
        onLoad={handleMapLoad}>
        {fileType === "geojson" && (
          <GeoJsonDataSource
            fileType={fileType}
            fileContent={normalizedFileContent}
            enableClustering={enableClustering}
            selectedFeatureId={mapFeature?.id}
          />
        )}
      </Map>
      {mapFeature && (
        <MapSidePanel
          mapFeature={mapFeature}
          setMapFeature={setMapFeature}
          onFlyToSelectedFeature={handleFlyToSelectedFeature}
        />
      )}
    </div>
  );
};

const MapSidePanel: React.FC<MapSidePanelProps> = ({
  mapFeature,
  setMapFeature,
  onFlyToSelectedFeature,
}) => {
  const t = useT();
  return (
    <div className="absolute top-4 right-4 z-10 h-4/6 w-80 overflow-auto rounded-md border-l bg-background opacity-97 shadow-lg">
      <div className="flex items-center justify-between border-b p-4">
        <div className="flex items-center gap-2">
          <IconButton
            onClick={onFlyToSelectedFeature}
            icon={<MapPinAreaIcon className="size-5" />}
          />
          <h2 className="text-lg font-semibold">{t("Feature Info")}</h2>
        </div>
        <Button
          variant={"ghost"}
          className="z-10 h-fit p-0 opacity-70 hover:bg-card hover:opacity-100 dark:font-thin"
          onClick={() => setMapFeature(null)}>
          <Cross2Icon className="size-5" />
        </Button>
      </div>

      <div className="max-h-full overflow-auto p-0 text-sm text-foreground">
        <div className="min-w-[24rem] divide-y divide-border border-t border-border">
          {Object.entries(mapFeature.properties || {})
            .filter(([key]) => key !== "id")
            .map(([key, value]) => (
              <div key={key} className="grid grid-cols-2 gap-2 px-4 py-2">
                <span className="font-medium break-words">{key}</span>
                <span className="w-fit text-right break-all whitespace-pre-wrap">
                  {Array.isArray(value)
                    ? value.join(", ")
                    : typeof value === "object" && value !== null
                      ? JSON.stringify(value, null, 2)
                      : String(value)}
                </span>
              </div>
            ))}
        </div>
      </div>
    </div>
  );
};

export { MapLibre };
