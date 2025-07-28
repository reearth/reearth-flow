import { MapPinAreaIcon } from "@phosphor-icons/react";
import { Cross2Icon } from "@radix-ui/react-icons";
import maplibregl from "maplibre-gl";
import * as React from "react";
import { useState, useMemo, useCallback } from "react";
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
  selectedFeature?:
    | any
    | { id?: string | number; properties?: { _originalId?: string | number } };
  convertedSelectedFeature?: any;
  mapRef: React.RefObject<maplibregl.Map | null>;
  onMapLoad: (onCenter?: boolean) => void;
  onSelectedFeature: (value: any) => void;
  onFlyToSelectedFeature?: (selectedFeature: any) => void;
};

type MapSidePanelProps = {
  selectedFeature: any;
  onShowFeaturePanel: (value: boolean) => void;
  onSelectedFeature: (value: any | null) => void;
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
  const [showFeaturePanel, setShowFeaturePanel] = useState<boolean>(false);
  const handleMapClick = useCallback(
    (e: maplibregl.MapLayerMouseEvent) => {
      if (e.features?.[0]) {
        onSelectedFeature(e.features[0]);
      } else {
        onSelectedFeature(undefined);
        setShowFeaturePanel(false);
      }
    },
    [onSelectedFeature, setShowFeaturePanel],
  );

  const handleMapDoubleClick = useCallback(
    (e: maplibregl.MapLayerMouseEvent) => {
      if (e.features?.[0]) {
        onSelectedFeature(e.features[0]);
        onFlyToSelectedFeature?.(e.features[0]);
        setShowFeaturePanel(true);
      }
    },
    [onSelectedFeature, onFlyToSelectedFeature, setShowFeaturePanel],
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
            fileType={fileType}
            fileContent={fileContent}
            enableClustering={enableClustering}
            selectedFeatureId={convertedSelectedFeature?.id}
          />
        )}
      </Map>
      {showFeaturePanel && convertedSelectedFeature && (
        <MapSidePanel
          selectedFeature={convertedSelectedFeature}
          onShowFeaturePanel={setShowFeaturePanel}
          onSelectedFeature={onSelectedFeature}
          onFlyToSelectedFeature={onFlyToSelectedFeature}
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
      ([key]) => key !== "_originalId",
    );
  }, [selectedFeature.properties]);

  return (
    <div className="absolute top-4 right-4 z-10 h-4/6 w-80 overflow-auto rounded-md border-l bg-background opacity-97 shadow-lg">
      <div className="flex items-center justify-between border-b p-4">
        <div className="flex items-center gap-2">
          <IconButton
            onClick={() => onFlyToSelectedFeature?.(selectedFeature)}
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
