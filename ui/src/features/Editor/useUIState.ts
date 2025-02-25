import { XYPosition } from "@xyflow/react";
import { MouseEvent, useCallback, useState } from "react";

import { ActionNodeType, Edge, Node } from "@flow/types";
import { cancellableDebounce } from "@flow/utils";

export default ({ hasReader }: { hasReader?: boolean }) => {
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
  const [nodePickerOpen, setNodePickerOpen] = useState<
    { position: XYPosition; nodeType: ActionNodeType } | undefined
  >(undefined);

  const handleNodePickerOpen = useCallback(
    (
      position?: XYPosition,
      nodeType?: ActionNodeType,
      isMainWorkflow?: boolean,
    ) => {
      if (isMainWorkflow === false && nodeType === "reader" && !hasReader) {
        return;
      }
      if (isMainWorkflow && nodeType === "reader" && hasReader) {
        return;
      }

      if (isMainWorkflow === false && nodeType === "writer") {
        return;
      }

      setNodePickerOpen(
        !position || !nodeType ? undefined : { position, nodeType },
      );
    },
    [hasReader],
  );

  const handleNodePickerClose = useCallback(
    () => setNodePickerOpen(undefined),
    [],
  );

  const [openPanel, setOpenPanel] = useState<
    "left" | "right" | "bottom" | undefined
  >(undefined);

  const handlePanelOpen = useCallback(
    (panel?: "left" | "right" | "bottom") => {
      if (!panel || openPanel === panel) {
        setOpenPanel(undefined);
      } else {
        setOpenPanel(panel);
      }
    },
    [openPanel],
  );

  const [rightPanelContent, setRightPanelContent] = useState<
    "version-history" | undefined
  >(undefined);

  const handleRightPanelOpen = useCallback(
    (content?: "version-history") => setRightPanelContent(content),
    [],
  );

  return {
    openPanel,
    nodePickerOpen,
    rightPanelContent,
    hoveredDetails,
    handleNodeHover,
    handleEdgeHover,
    handlePanelOpen,
    handleNodePickerOpen,
    handleNodePickerClose,
    handleRightPanelOpen,
  };
};
