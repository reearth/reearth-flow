import { XYPosition } from "@xyflow/react";
import { useCallback, useState } from "react";

import type { ActionNodeType } from "@flow/types";

export default () => {
  const [nodePickerOpen, setNodePickerOpen] = useState<
    { position: XYPosition; nodeType: ActionNodeType } | undefined
  >(undefined);

  const handleNodePickerOpen = useCallback(
    (
      position?: XYPosition,
      nodeType?: ActionNodeType,
      isMainWorkflow?: boolean,
    ) => {
      if (isMainWorkflow === false && nodeType === "reader") {
        return;
      }

      if (isMainWorkflow === false && nodeType === "writer") {
        return;
      }

      setNodePickerOpen(
        !position || !nodeType ? undefined : { position, nodeType },
      );
    },
    [],
  );

  const handleNodePickerClose = useCallback(
    () => setNodePickerOpen(undefined),
    [],
  );

  const [rightPanelContent, setRightPanelContent] = useState<
    "version-history" | undefined
  >(undefined);

  const handleRightPanelOpen = useCallback(
    (content?: "version-history") => setRightPanelContent(content),
    [],
  );

  return {
    nodePickerOpen,
    rightPanelContent,
    handleNodePickerOpen,
    handleNodePickerClose,
    handleRightPanelOpen,
  };
};
