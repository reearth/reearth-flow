import { useReactFlow, XYPosition } from "@xyflow/react";
import { useCallback } from "react";

export default () => {
  const { setCenter } = useReactFlow();

  const handleOnNodeFocus = useCallback(
    (position: XYPosition, zoom: number) => {
      if (position) {
        setCenter(position.x, position.y, { zoom });
      }
    },
    [setCenter],
  );
  return {
    handleOnNodeFocus,
  };
};
