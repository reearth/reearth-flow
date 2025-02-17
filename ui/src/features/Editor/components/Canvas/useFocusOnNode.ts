import { useReactFlow, XYPosition } from "@xyflow/react";
import { useCallback } from "react";

export default () => {
  const { setCenter } = useReactFlow();

  const handleOnNodeFocus = useCallback(
    (
      position: XYPosition,
      measured: { width: number; height: number },
      zoom: number,
    ) => {
      if (position && measured) {
        const x = position.x + measured.width / 2;
        const y = position.y + measured.height / 2;
        setCenter(x, y, { zoom });
      }
    },
    [setCenter],
  );
  return {
    handleOnNodeFocus,
  };
};
