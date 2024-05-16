import { CoreVisualizer } from "@reearth/core";
import { useEffect, useState } from "react";

import { Button } from "@flow/components";
import fires from "@flow/mock_data/fires.json";

type Props = {};

type SceneMode = "3d" | "2d" | "columbus";

const sceneModes: SceneMode[] = ["2d", "3d"];

const Map: React.FC<Props> = () => {
  const [isReady, setIsReady] = useState(false);
  const [sceneMode, setSceneMode] = useState<SceneMode>("2d");

  useEffect(() => {
    if (isReady) return;
    setIsReady(true);
  }, [isReady]);

  return (
    <div className="relative w-6/12">
      <div className="flex flex-col flex-wrap bg-zinc-900/50 border border-zinc-700 rounded-md text-zinc-400 transition-all top-2 left-2 absolute z-10">
        {sceneModes.map(b => (
          <Button
            className={`transition-all text-zinc-400 hover:bg-zinc-700 hover:text-zinc-100 cursor-pointer ${sceneMode === b ? "bg-zinc-800 text-zinc-300" : ""}`}
            variant="ghost"
            size="icon"
            key={b}
            onClick={() => setSceneMode(b)}>
            {b}
          </Button>
        ))}
      </div>
      <CoreVisualizer
        ready={isReady}
        engine="cesium"
        sceneProperty={{
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
        }}
        layers={[
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
        ]}
      />
    </div>
  );
};

export { Map };
