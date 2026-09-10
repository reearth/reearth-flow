import { XYPosition } from "@xyflow/react";
import { useCallback, useState } from "react";

import { useFollowSync } from "@flow/lib/yjs";
import type { ActionNodeType, AwarenessNodePicker } from "@flow/types";

export default ({
  onUserFocusedElement,
  onNodePickerAwareness,
  isFollowing = false,
  followNodePicker,
}: {
  onUserFocusedElement?: (isOpen: boolean) => void;
  onNodePickerAwareness?: (nodePicker: AwarenessNodePicker | null) => void;
  isFollowing?: boolean;
  followNodePicker?: AwarenessNodePicker | null;
}) => {
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
      const next = !position || !nodeType ? undefined : { position, nodeType };
      setNodePickerOpen(next);
      if (openViaShortcut) {
        setOpenNodePickerViaShortcut(true);
      }
      onUserFocusedElement?.(true);
      onNodePickerAwareness?.(
        next ? { nodeType: next.nodeType, position: next.position } : null,
      );
    },
    [onUserFocusedElement, onNodePickerAwareness],
  );

  const handleNodePickerClose = useCallback(() => {
    setNodePickerOpen(undefined);
    setOpenNodePickerViaShortcut(false);
    onUserFocusedElement?.(false);
    onNodePickerAwareness?.(null);
  }, [onUserFocusedElement, onNodePickerAwareness]);

  const handleFollowNodePicker = useCallback(
    (next: AwarenessNodePicker | null) => {
      if (!next) {
        handleNodePickerClose();
        return;
      }
      handleNodePickerOpen(next.position, next.nodeType as ActionNodeType);
    },
    [handleNodePickerOpen, handleNodePickerClose],
  );

  useFollowSync<AwarenessNodePicker>({
    active: isFollowing,
    follow: followNodePicker ?? null,
    local: nodePickerOpen
      ? { nodeType: nodePickerOpen.nodeType, position: nodePickerOpen.position }
      : null,
    onFollow: handleFollowNodePicker,
    isSame: (a, b) =>
      a.nodeType === b.nodeType &&
      a.position.x === b.position.x &&
      a.position.y === b.position.y,
  });

  return {
    nodePickerOpen,
    openNodePickerViaShortcut,
    handleNodePickerOpen,
    handleNodePickerClose,
  };
};
