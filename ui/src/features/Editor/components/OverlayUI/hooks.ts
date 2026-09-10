import { useCallback, useState } from "react";

import { useFollowSync } from "@flow/lib/yjs";
import type { AwarenessDialog } from "@flow/types";

import { DialogOptions } from "./types";

/** Dialogs whose open state lives in this hook rather than in the Homebar. */
const OWNED_DIALOGS: readonly AwarenessDialog[] = [
  "deploy",
  "share",
  "version",
  "debugStop",
  "layout",
];

/**
 * Subset that spotlight follows. Deploy, share and debugStop are excluded on
 * purpose — they commit changes, so opening them on a follower's screen is not
 * a passive observation.
 */
const FOLLOWABLE_DIALOGS: readonly AwarenessDialog[] = ["version", "layout"];

export default ({
  onUserFocusedElement,
  onDialogAwareness,
  isFollowing = false,
  followDialog,
}: {
  onUserFocusedElement?: (isOpen: boolean) => void;
  onDialogAwareness?: (
    dialog: AwarenessDialog | null,
    ownedDialogs: readonly AwarenessDialog[],
  ) => void;
  isFollowing?: boolean;
  followDialog?: AwarenessDialog | null;
}) => {
  const [showDialog, setShowDialog] = useState<DialogOptions>(undefined);

  const handleDialogOpen = useCallback(
    (dialog: DialogOptions) => {
      setShowDialog(dialog);
      onUserFocusedElement?.(true);
      onDialogAwareness?.(dialog ?? null, OWNED_DIALOGS);
    },
    [onUserFocusedElement, onDialogAwareness],
  );

  const handleDialogClose = useCallback(() => {
    setShowDialog(undefined);
    onUserFocusedElement?.(false);
    onDialogAwareness?.(null, OWNED_DIALOGS);
  }, [onUserFocusedElement, onDialogAwareness]);

  const handleFollowDialog = useCallback(
    (dialog: AwarenessDialog | null) =>
      dialog ? handleDialogOpen(dialog) : handleDialogClose(),
    [handleDialogOpen, handleDialogClose],
  );

  const followable =
    followDialog && FOLLOWABLE_DIALOGS.includes(followDialog)
      ? followDialog
      : null;

  useFollowSync<AwarenessDialog>({
    active: isFollowing,
    follow: followable,
    local: showDialog && OWNED_DIALOGS.includes(showDialog) ? showDialog : null,
    onFollow: handleFollowDialog,
  });

  return {
    showDialog,
    handleDialogOpen,
    handleDialogClose,
  };
};
