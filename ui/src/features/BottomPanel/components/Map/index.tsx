import { CoreVisualizer } from "@reearth/core";

import fires from "@flow/mock_data/fires.json";

type Props = {};

const Map: React.FC<Props> = () => {
  return (
    <div className="relative w-6/12">
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
