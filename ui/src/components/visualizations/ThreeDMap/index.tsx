import { CoreVisualizer, Layer, SceneProperty } from "@reearth/core";
import { useEffect, useState } from "react";

import fires from "@flow/mock_data/fires.json";

const ThreeDMap: React.FC = () => {
  const [isReady, setIsReady] = useState(false);

  const sceneProperty: SceneProperty = {
    default: {
      sceneMode: "3d",
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
    <CoreVisualizer
      engine="cesium"
      isBuilt
      ready={isReady}
      sceneProperty={sceneProperty}
      layers={layers}
    />
  );
};

export { ThreeDMap };
