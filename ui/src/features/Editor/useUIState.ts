import { XYPosition } from "@xyflow/react";
import { useCallback, useState } from "react";

import type { ActionNodeType } from "@flow/types";

export default () => {
  const [nodePickerOpen, setNodePickerOpen] = useState<
    { position: XYPosition; nodeType: ActionNodeType } | undefined
  >(undefined);
  const [openNodePickerViaShortcut, setOpenNodePickerViaShortcut] =
    useState<boolean>(false);
  const handleNodePickerOpen = useCallback(
    (
      position?: XYPosition,
      nodeType?: ActionNodeType,
      openViaShortcut?: boolean,
    ) => {
      setNodePickerOpen(
        !position || !nodeType ? undefined : { position, nodeType },
      );
      if (openViaShortcut) {
        setOpenNodePickerViaShortcut(true);
      }
    },
    [],
  );

  const handleNodePickerClose = useCallback(() => {
    setNodePickerOpen(undefined);
    setOpenNodePickerViaShortcut(false);
  }, []);

  return {
    nodePickerOpen,
    openNodePickerViaShortcut,
    handleNodePickerOpen,
    handleNodePickerClose,
  };
};
