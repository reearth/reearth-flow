import { useCallback, useEffect, useState } from "react";
import type { Map as YMap } from "yjs";

import type { YWorkflow } from "./types";
import {
  hasLegacyPorts,
  migrateLegacyPorts,
} from "./utils/legacyPortsMigration";

export default ({
  yWorkflows,
  onProjectSnapshotSave,
}: {
  yWorkflows: YMap<YWorkflow>;
  onProjectSnapshotSave: () => void;
}) => {
  const [showLegacyPortsDialog, setShowLegacyPortsDialog] = useState(false);
  const [dismissed, setDismissed] = useState(false);
  useEffect(() => {
    setShowLegacyPortsDialog(hasLegacyPorts(yWorkflows));
    const update = () => {
      setShowLegacyPortsDialog(!dismissed && hasLegacyPorts(yWorkflows));
    };

    update();
    yWorkflows.observeDeep(update);
    return () => {
      yWorkflows.unobserveDeep(update);
    };
  }, [yWorkflows, dismissed]);

  const handleLegacyPortsMigrate = useCallback(() => {
    // Perform the migration without adding to undo stack, as this is a one-time migration that should not be undoable.
    yWorkflows.doc?.transact(() => {
      migrateLegacyPorts(yWorkflows);
    }, "legacy-ports-migration");
    onProjectSnapshotSave();
    setShowLegacyPortsDialog(false);
    setDismissed(true);
  }, [yWorkflows, onProjectSnapshotSave]);

  const handleLegacyPortsDialogClose = useCallback(() => {
    setShowLegacyPortsDialog(false);
    setDismissed(true);
  }, []);

  return {
    showLegacyPortsDialog,
    handleLegacyPortsMigrate,
    handleLegacyPortsDialogClose,
  };
};
