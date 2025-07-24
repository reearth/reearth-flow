import { MapPinAreaIcon } from "@phosphor-icons/react";
import { Cross2Icon } from "@radix-ui/react-icons";
import bbox from "@turf/bbox";
import maplibregl, { LngLatBounds } from "maplibre-gl";
import * as React from "react";
import { useRef, useEffect, useState, useMemo, useCallback } from "react";
import { Map as MapLibreMap } from "react-map-gl/maplibre";

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
  selectedFeature?:
    | any
    | { id?: string | number; properties?: { originalId?: string | number } };
  shouldFlyToFeature?: boolean;
  fitDataToBounds?: boolean;
  onSelectedFeature: (value: any) => void;
  onFitDataToBoundsChange?: (value: boolean) => void;
  onShouldFlyToFeatureChange?: (value: boolean) => void;
};

type MapSidePanelProps = {
  selectedFeature: any;
  onShowFeaturePanel: (value: boolean) => void;
  onSelectedFeature: (value: any | null) => void;
  onFlyToSelectedFeature?: () => void;
};

const MapLibre: React.FC<Props> = ({
  className,
  fileContent,
  fileType,
  enableClustering,
  selectedFeature,
  shouldFlyToFeature,
  fitDataToBounds,
  onSelectedFeature,
  onFitDataToBoundsChange,
  onShouldFlyToFeatureChange,
}) => {
  const mapRef = useRef<maplibregl.Map | null>(null);
  const [showFeaturePanel, setShowFeaturePanel] = useState<boolean>(false);

  const featureMap = useMemo(() => {
    if (!fileContent?.features) return null;

    return new Map(
      fileContent.features
        .map((f: any) => {
          const id = f.id ?? f.properties?.originalId;
          return id !== undefined ? ([id, f] as [string | number, any]) : null;
        })
        .filter(Boolean),
    );
  }, [fileContent?.features]);

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

  const convertedSelectedFeature = useMemo(() => {
    if (!selectedFeature || !featureMap) return null;

    if ("geometry" in selectedFeature && selectedFeature.geometry) {
      return selectedFeature;
    }

    const featureId =
      selectedFeature.id ?? selectedFeature.properties?.originalId;
    if (featureId === undefined) return null;

    let normalizedId = featureId;
    if (typeof featureId === "string") {
      normalizedId = JSON.parse(featureId);
      try {
        normalizedId = JSON.parse(featureId);
      } catch {
        normalizedId = featureId;
      }
    }

    return featureMap.get(normalizedId) || null;
  }, [selectedFeature, featureMap]);

  const handleMapLoad = useCallback(
    (onCenter?: boolean) => {
      if (mapRef.current && dataBounds) {
        mapRef.current.fitBounds(dataBounds, {
          padding: 40,
          duration: onCenter ? 500 : 0,
          maxZoom: 16,
        });
      }
    },
    [dataBounds],
  );

  const handleFlyToSelectedFeature = useCallback(() => {
    if (mapRef.current && convertedSelectedFeature) {
      try {
        const [minLng, minLat, maxLng, maxLat] = bbox(convertedSelectedFeature);
        mapRef.current.fitBounds(
          [
            [minLng, minLat],
            [maxLng, maxLat],
          ],

          { padding: 40, duration: 500, maxZoom: 24 },
        );
      } catch (err) {
        console.error("Error computing bbox for selectedFeature:", err);
      }
    }
  }, [convertedSelectedFeature]);

  useEffect(() => {
    if (convertedSelectedFeature && shouldFlyToFeature) {
      handleFlyToSelectedFeature();
      onShouldFlyToFeatureChange?.(false);
    }
  }, [
    convertedSelectedFeature,
    shouldFlyToFeature,
    handleFlyToSelectedFeature,
    onShouldFlyToFeatureChange,
  ]);

  useEffect(() => {
    if (fitDataToBounds) {
      handleMapLoad(true);
      onFitDataToBoundsChange?.(false);
    }
  }, [fitDataToBounds, onFitDataToBoundsChange, handleMapLoad]);

  return (
    <div className={`relative size-full ${className}`}>
      <MapLibreMap
        ref={(instance) => {
          if (instance) mapRef.current = instance.getMap();
        }}
        mapStyle="https://basemaps.cartocdn.com/gl/positron-gl-style/style.json"
        style={{ width: "100%", height: "100%" }}
        maplibreLogo={true}
        interactiveLayerIds={["point-layer", "line-layer", "polygon-layer"]}
        onClick={(e) => {
          if (e.features?.[0]) {
            onSelectedFeature(e.features[0]);
            setShowFeaturePanel(true);
          }
        }}
        onDblClick={(e) => {
          if (e.features?.[0]) {
            onSelectedFeature(e.features[0]);
            onShouldFlyToFeatureChange?.(true);
            setShowFeaturePanel(true);
          }
        }}
        onLoad={() => handleMapLoad()}>
        {fileType === "geojson" && (
          <GeoJsonDataSource
            key={`geojson-source-${enableClustering}`}
            fileType={fileType}
            fileContent={fileContent}
            enableClustering={enableClustering}
            selectedFeatureId={convertedSelectedFeature?.id}
          />
        )}
      </MapLibreMap>
      {showFeaturePanel && convertedSelectedFeature && (
        <MapSidePanel
          selectedFeature={convertedSelectedFeature}
          onShowFeaturePanel={setShowFeaturePanel}
          onSelectedFeature={onSelectedFeature}
          onFlyToSelectedFeature={handleFlyToSelectedFeature}
        />
      )}
    </div>
  );
};

const MapSidePanel: React.FC<MapSidePanelProps> = ({
  selectedFeature,
  onShowFeaturePanel,
  onSelectedFeature,
  onFlyToSelectedFeature,
}) => {
  const t = useT();
  const handleClosePanel = useCallback(() => {
    onShowFeaturePanel(false);
    onSelectedFeature(null);
  }, [onSelectedFeature, onShowFeaturePanel]);

  const featureProperties = useMemo(() => {
    return Object.entries(selectedFeature.properties || {}).filter(
      ([key]) => key !== "originalId",
    );
  }, [selectedFeature.properties]);

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
          onClick={handleClosePanel}>
          <Cross2Icon className="size-5" />
        </Button>
      </div>

      <div className="max-h-full overflow-auto p-0 text-sm text-foreground">
        <div className="min-w-[24rem] divide-y divide-border border-t border-border">
          {featureProperties.map(([key, value]) => (
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
