import { Cartesian3 } from "cesium";
import { useEffect } from "react";
import { Entity, useCesium } from "resium";

// const startingPosition = Cartesian3.fromDegrees(
//   137.12970211846854,
//   37.13273015737172,
//   2899856.004369806,
// );
const entityPosition = Cartesian3.fromDegrees(
  137.12970211846854,
  37.13273015737172,
  10,
);
const pointGraphics = { pixelSize: 10 };

type Props = {
  isLoaded: boolean;
};

const CesiumContents: React.FC<Props> = ({ isLoaded }) => {
  const { viewer } = useCesium();

  useEffect(() => {
    if (isLoaded && viewer) {
      viewer.camera.setView({
        destination: Cartesian3.fromDegrees(138.2529, 36.2048, 1000000),
        orientation: {
          heading: 0.0,
          pitch: -1.3,
          roll: 0.0,
        },
      });
    }
  }, [isLoaded, viewer]);
  return <Entity position={entityPosition} point={pointGraphics} />;
};

export { CesiumContents };
