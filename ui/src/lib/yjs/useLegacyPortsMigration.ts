import { useCallback, useEffect, useState } from "react";
import type { Map as YMap } from "yjs";

import type { YWorkflow } from "./types";
import {
  hasLegacyPorts,
  migrateLegacyPorts,
} from "./utils/legacyPortsMigration";

export default ({ yWorkflows }: { yWorkflows: YMap<YWorkflow> }) => {
  const [showLegacyPortsDialog, setShowLegacyPortsDialog] = useState(false);

  useEffect(() => {
    setShowLegacyPortsDialog(hasLegacyPorts(yWorkflows));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleLegacyPortsMigrate = useCallback(() => {
    // Perform the migration without adding to undo stack, as this is a one-time migration that should not be undoable.
    yWorkflows.doc?.transact(() => {
      migrateLegacyPorts(yWorkflows);
    }, "legacy-ports-migration");
    setShowLegacyPortsDialog(false);
  }, [yWorkflows]);

  const handleLegacyPortsDialogClose = useCallback(() => {
    setShowLegacyPortsDialog(false);
  }, []);

  return {
    showLegacyPortsDialog,
    handleLegacyPortsMigrate,
    handleLegacyPortsDialogClose,
  };
};
