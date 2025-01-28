import { MouseEvent, useCallback, useState } from "react";

import { Edge, Node } from "@flow/types";
import { cancellableDebounce } from "@flow/utils";

export default () => {
  const [hoveredDetails, setHoveredDetails] = useState<
    Node | Edge | undefined
  >();

  const hoverActionDebounce = cancellableDebounce(
    (callback: () => void) => callback(),
    100,
  );

  const handleNodeHover = useCallback(
    (e: MouseEvent, node?: Node) => {
      hoverActionDebounce.cancel();
      if (e.type === "mouseleave" && hoveredDetails) {
        hoverActionDebounce(() => setHoveredDetails(undefined));
      } else {
        setHoveredDetails(node);
      }
    },
    [hoveredDetails, hoverActionDebounce],
  );

  const handleEdgeHover = useCallback(
    (e: MouseEvent, edge?: Edge) => {
      if (e.type === "mouseleave" && hoveredDetails) {
        setHoveredDetails(undefined);
      } else {
        setHoveredDetails(edge);
      }
    },
    [hoveredDetails],
  );

  return {
    hoveredDetails,
    handleNodeHover,
    handleEdgeHover,
  };
};
