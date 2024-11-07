import { CoreVisualizer, SceneProperty, Layer } from "@reearth/core";
import { useEffect, useState } from "react";

import { Button, TwoDMap } from "@flow/components";
import fires from "@flow/mock_data/fires.json";

export type MapMode = "2d" | "3d";

const mapModes: MapMode[] = ["2d", "3d"];

type Props = {
  mapMode: MapMode;
  setMapMode?: (mode: MapMode) => void;
};

const Map: React.FC<Props> = ({ mapMode, setMapMode }) => {
  const [isReady, setIsReady] = useState(false);

  const sceneProperty: SceneProperty = {
    default: {
      sceneMode: mapMode,
    },
    camera: {
      camera: {
        lng: 127.05177672074426,
        lat: -6.260283141094241,
        height: 7594277.78896907,
        heading: 1.129814464206902e-9,
        pitch: -1.5707963267948966,
        roll: 0,
        fov: 1,
        aspectRatio: 1,
      },
    },
    tiles: [
      {
        id: "default",
        tile_type: "default",
      },
    ],
  };

  const layers: Layer[] = [
    {
      id: "marker",
      type: "simple",
      data: {
        type: "geojson",
        value: fires,
      },
      marker: {
        imageColor: {
          expression: {
            conditions: [["true", "color('#FF0000')"]],
          },
        },
      },
    },
  ];

  useEffect(() => {
    if (isReady) return;
    setIsReady(true);
  }, [isReady]);

  return (
    <div className="flex w-1/2">
      <div className="relative w-full">
        <div className="absolute left-2 top-2 z-10 flex flex-col flex-wrap rounded-md border bg-background transition-all">
          {mapModes.map((b) => (
            <Button
              className={`cursor-pointer rounded-none transition-all ${mapMode === b ? "bg-accent text-accent-foreground" : ""}`}
              variant="ghost"
              size="icon"
              key={b}
              onClick={() => mapMode !== b && setMapMode?.(b)}>
              {b}
            </Button>
          ))}
        </div>
        {mapMode === "2d" ? (
          <TwoDMap />
        ) : (
          <CoreVisualizer
            engine="cesium"
            isBuilt
            ready={isReady}
            sceneProperty={sceneProperty}
            layers={layers}
          />
        )}
      </div>
    </div>
  );
};

export { Map };
