import { MapPinAreaIcon } from "@phosphor-icons/react";
import { Cross2Icon } from "@radix-ui/react-icons";
import maplibregl, { LngLatBounds } from "maplibre-gl";
import * as React from "react";
import { useRef, useState, useMemo, useCallback } from "react";
import { Map, Source, Marker, Layer, LayerProps } from "react-map-gl/maplibre";

import "maplibre-gl/dist/maplibre-gl.css";

import { Button, IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

type Props = {
  className?: string;
  fileContent: any | null;
  fileType: SupportedDataTypes | null;
};

type MapSidePanelProps = {
  selectedFeature: any;
  setSelectedFeature: (value: any) => void;
  onFlyToSelectedFeature?: (selectedFeature: any) => void;
};

const polygonLayer: LayerProps = {
  type: "fill",
  paint: {
    "fill-color": "#3f3f45",
    "fill-opacity": 0.8,
  },
};

const lineStringLayer: LayerProps = {
  type: "line",
  paint: {
    "line-color": "#3f3f45",
    "line-width": 2,
  },
};

const MapLibre: React.FC<Props> = ({ className, fileContent, fileType }) => {
  const mapRef = useRef<maplibregl.Map | null>(null);
  const [selectedFeature, setSelectedFeature] = useState<any>(null);

  const dataBounds = useMemo(() => {
    if (!fileContent?.features?.length) return null;

    const bounds = new LngLatBounds();

    fileContent.features.forEach((feature: any) => {
      const coords = feature.geometry?.coordinates;
      if (!coords) return;

      if (feature.geometry.type === "Point") {
        bounds.extend(coords);
      } else if (Array.isArray(coords[0])) {
        coords.flat(Infinity).forEach((c: any) => {
          if (Array.isArray(c) && c.length >= 2) {
            bounds.extend([c[0], c[1]] as [number, number]);
          }
        });
      }
    });

    return bounds.isEmpty() ? null : bounds;
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
      const coords = selectedFeature.geometry?.coordinates;
      if (coords && selectedFeature.geometry.type === "Point") {
        mapRef.current.flyTo({
          center: coords,
          zoom: 16,
          duration: 300,
        });
      }
    }
  }, [selectedFeature]);

  return (
    <div className={`relative size-full ${className}`}>
      <Map
        ref={(instance) => {
          if (instance) mapRef.current = instance.getMap();
        }}
        mapStyle="https://basemaps.cartocdn.com/gl/positron-gl-style/style.json"
        style={{ width: "100%", height: "100%" }}
        maplibreLogo={true}
        onLoad={handleMapLoad}>
        <Source type={fileType || "geojson"} data={fileContent}>
          {fileContent?.features?.some(
            (feature: GeoJSON.Feature) =>
              feature.geometry.type === "LineString",
          ) && (
            <Layer
              {...lineStringLayer}
              filter={["==", ["geometry-type"], "LineString"]}
            />
          )}

          {fileContent?.features?.some(
            (feature: GeoJSON.Feature) => feature.geometry.type === "Polygon",
          ) && (
            <Layer
              {...polygonLayer}
              filter={["==", ["geometry-type"], "Polygon"]}
            />
          )}

          {fileContent?.features?.some(
            (feature: GeoJSON.Feature) => feature.geometry.type === "Point",
          ) &&
            fileContent.features.map((feature: any, i: number) => {
              const coords = feature.geometry?.coordinates;
              if (!coords || feature.geometry.type !== "Point") return null;

              return (
                <Marker
                  key={i}
                  color="#3f3f45"
                  longitude={coords[0]}
                  latitude={coords[1]}
                  onClick={(e) => {
                    e.originalEvent.stopPropagation();
                    setSelectedFeature(feature);
                  }}
                />
              );
            })}
        </Source>
      </Map>

      {selectedFeature && (
        <MapSidePanel
          selectedFeature={selectedFeature}
          setSelectedFeature={setSelectedFeature}
          onFlyToSelectedFeature={handleFlyToSelectedFeature}
        />
      )}
    </div>
  );
};

const MapSidePanel: React.FC<MapSidePanelProps> = ({
  selectedFeature,
  setSelectedFeature,
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
          onClick={() => setSelectedFeature(null)}>
          <Cross2Icon className="size-5" />
        </Button>
      </div>

      <div className="max-h-full overflow-auto p-0 text-sm text-foreground">
        <div className="min-w-[24rem] divide-y divide-border border-t border-border">
          {Object.entries(selectedFeature.properties || {}).map(
            ([key, value]) => (
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
            ),
          )}
        </div>
      </div>
    </div>
  );
};

export { MapLibre };
