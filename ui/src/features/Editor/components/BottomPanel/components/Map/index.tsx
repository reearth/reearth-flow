import { CoreVisualizer, SceneMode, SceneProperty, Layer } from "@reearth/core";
import { useEffect, useState } from "react";

import { Button } from "@flow/components";
import fires from "@flow/mock_data/fires.json";

const sceneModes: SceneMode[] = ["2d", "3d"];

const Map: React.FC = () => {
  const [isReady, setIsReady] = useState(false);
  const [sceneMode, setSceneMode] = useState<SceneMode>("2d");

  const sceneProperty: SceneProperty = {
    default: {
      sceneMode,
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
    <div className="relative w-6/12">
      <div className="absolute left-2 top-2 z-10 flex flex-col flex-wrap rounded-md border bg-background transition-all">
        {sceneModes.map(b => (
          <Button
            className={`cursor-pointer rounded-none transition-all ${sceneMode === b ? "bg-accent text-accent-foreground" : ""}`}
            variant="ghost"
            size="icon"
            key={b}
            onClick={() => sceneMode !== b && setSceneMode(b)}>
            {b}
          </Button>
        ))}
      </div>
      <CoreVisualizer
        engine="cesium"
        isBuilt
        ready={isReady}
        sceneProperty={sceneProperty}
        layers={layers}
      />
    </div>
  );
};

export { Map };
