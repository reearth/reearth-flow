import { Cross2Icon } from "@radix-ui/react-icons";
import maplibregl, { LngLatBounds } from "maplibre-gl";
import * as React from "react";
import { useRef, useState, useMemo, useCallback } from "react";
import { Map, Source, Marker } from "react-map-gl/maplibre";

import "maplibre-gl/dist/maplibre-gl.css";

import { Button } from "@flow/components";
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
        <Source id="data" type={fileType || "geojson"} data={fileContent} />
        {fileContent.features.map((feature: any, i: number) => {
          const coords = feature.geometry?.coordinates;
          if (!coords || feature.geometry.type !== "Point") return null;

          return (
            <Marker
              key={i}
              color="#4169E1"
              longitude={coords[0]}
              latitude={coords[1]}
              onClick={(e) => {
                e.originalEvent.stopPropagation();
                setSelectedFeature(feature);
              }}
            />
          );
        })}
      </Map>

      {selectedFeature && (
        <MapSidePanel
          selectedFeature={selectedFeature}
          setSelectedFeature={setSelectedFeature}
        />
      )}
    </div>
  );
};

const MapSidePanel: React.FC<MapSidePanelProps> = ({
  selectedFeature,
  setSelectedFeature,
}) => {
  const t = useT();
  return (
    <div className="absolute top-0 right-0 w-80 h-full bg-background border-l shadow-lg z-10 overflow-auto animate-in slide-in-from-right duration-300">
      <div className="p-4 flex justify-between items-center border-b">
        <h2 className="font-semibold text-lg">{t("Feature Info")}</h2>
        <Button
          variant={"ghost"}
          className="h-fit p-0 opacity-70 dark:font-thin hover:bg-card hover:opacity-100 z-10"
          onClick={() => setSelectedFeature(null)}>
          <Cross2Icon className="size-5" />
        </Button>
      </div>

      <div className="p-0 text-sm text-foreground overflow-auto max-h-full">
        <div className="min-w-[24rem] divide-y divide-border border-t border-border">
          {Object.entries(selectedFeature.properties || {}).map(
            ([key, value]) => (
              <div key={key} className="grid grid-cols-2 gap-2 px-4 py-2">
                <div className="font-medium break-words">{key}</div>
                <div className="text-right w-fit break-all whitespace-pre-wrap">
                  {Array.isArray(value)
                    ? value.join(", ")
                    : typeof value === "object" && value !== null
                      ? JSON.stringify(value, null, 2)
                      : String(value)}
                </div>
              </div>
            ),
          )}
        </div>
      </div>
    </div>
  );
};

export { MapLibre };
