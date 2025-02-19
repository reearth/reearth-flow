import { XYPosition } from "@xyflow/react";
import { useCallback, useState } from "react";

import { ActionNodeType } from "@flow/types";

export default ({ hasReader }: { hasReader?: boolean }) => {
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
