import { useCallback, useEffect, useState } from "react";
import type { Map as YMap } from "yjs";

import type { YWorkflow } from "./types";
import {
  hasLegacyActionNames,
  migrateLegacyActionNames,
} from "./utils/legacyActionNamesMigration";
import {
  isLegacyMigrationComplete,
  markLegacyMigrationComplete,
} from "./utils/legacyMigrationVersion";
import {
  hasLegacyPorts,
  migrateLegacyPorts,
} from "./utils/legacyPortsMigration";

// The version stamp check comes first: once a doc is stamped (migrated here,
// imported migrated, or created fresh) the full workflow scans are skipped
// on every doc change.
const needsLegacyMigration = (yWorkflows: YMap<YWorkflow>) =>
  !isLegacyMigrationComplete(yWorkflows.doc) &&
  (hasLegacyPorts(yWorkflows) || hasLegacyActionNames(yWorkflows));

export default ({
  yWorkflows,
  onProjectSnapshotSave,
}: {
  yWorkflows: YMap<YWorkflow>;
  onProjectSnapshotSave: () => void;
}) => {
  const [showLegacyMigrationDialog, setShowLegacyMigrationDialog] =
    useState(false);
  const [dismissed, setDismissed] = useState(false);
  useEffect(() => {
    const update = () => {
      setShowLegacyMigrationDialog(
        !dismissed && needsLegacyMigration(yWorkflows),
      );
    };

    update();
    yWorkflows.observeDeep(update);
    // The stamp lives in doc metadata, so a collaborator stamping the doc
    // hides the dialog here even without a workflow change.
    const yMetadata = yWorkflows.doc?.getMap("metadata");
    yMetadata?.observe(update);
    return () => {
      yWorkflows.unobserveDeep(update);
      yMetadata?.unobserve(update);
    };
  }, [yWorkflows, dismissed]);

  const handleLegacyMigration = useCallback(() => {
    // Perform the migration without adding to undo stack, as this is a one-time migration that should not be undoable.
    yWorkflows.doc?.transact(() => {
      // Ports first: the port migration keys off pre-rename router names to
      // decide which "default" handles to preserve.
      migrateLegacyPorts(yWorkflows);
      migrateLegacyActionNames(yWorkflows);
      markLegacyMigrationComplete(yWorkflows.doc);
    }, "legacy-migration");
    onProjectSnapshotSave();
    setShowLegacyMigrationDialog(false);
    setDismissed(true);
  }, [yWorkflows, onProjectSnapshotSave]);

  const handleLegacyMigrationDialogClose = useCallback(() => {
    setShowLegacyMigrationDialog(false);
    setDismissed(true);
  }, []);

  return {
    showLegacyMigrationDialog,
    handleLegacyMigration,
    handleLegacyMigrationDialogClose,
  };
};
