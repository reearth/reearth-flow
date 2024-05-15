import { CoreVisualizer } from "@reearth/core";

import { Button } from "@flow/components";
import fires from "@flow/mock_data/fires.json";

type Props = {};

const Map: React.FC<Props> = () => {
  const buttons = ["2D", "3D"];
  return (
    <div className="relative w-6/12">
      <div className="flex flex-col flex-wrap bg-zinc-900/50 border border-zinc-700 rounded-md text-zinc-400 transition-all top-2 left-2 absolute z-10 p-1">
        {buttons.map(b => (
          <Button
            className={`transition-all text-zinc-400 hover:bg-zinc-700 hover:text-zinc-100 cursor-pointer `}
            variant="ghost"
            size="icon"
            key={b}>
            {b}
          </Button>
        ))}
      </div>
      <CoreVisualizer
        ready={true}
        engine="cesium"
        sceneProperty={{
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
