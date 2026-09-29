import { useCallback, useEffect, useMemo, useRef } from "react";
import { useY } from "react-yjs";
import { Map as YMap } from "yjs";
import type { Doc, Transaction, UndoManager, YEvent, YMapEvent } from "yjs";

import {
  collectIgnoredNodeIds,
  computeDeploymentFingerprint,
  isDeploymentRelevantEvent,
} from "@flow/lib/yjs/deploymentTracking";
import type { YWorkflow } from "@flow/lib/yjs/types";
import type { DeploymentTracking, YDocMetadataValue } from "@flow/types";

const emptyMetadata = new YMap<YDocMetadataValue>();

const TRACKING_KEY = "deploymentTracking";
const CHANGED_KEY = "deploymentChangedVersion";

export type DeploymentChangeStatus = "changed" | "unchanged";

const isDeploymentTracking = (value: unknown): value is DeploymentTracking =>
  typeof value === "object" &&
  value !== null &&
  "version" in value &&
  "fingerprint" in value;

// Tracks whether the canvas has changed since the project was last deployed
// from the editor. The state lives in the Yjs doc's metadata map, so every
// collaborator shares it, it persists with the project, and the UndoManager
// (scoped to the workflows map) never touches it.
//
// Deploys and edits write different keys, so neither can overwrite the other:
// a deploy writes the record, an edit writes the version it moved away from.
// A new deploy clears the flag without writing it, because the recorded
// version no longer matches.
//
// It is a latch. The first relevant local edit marks it changed, and after that
// every edit returns after two map reads. The graph fingerprint is compared
// again only while it reads unchanged: on undo/redo, which can restore the
// deployed state, and when a remote graph change or deploy record arrives,
// since the collaborator's own flag can be lost to a concurrent deploy.
export default ({
  yDoc,
  yWorkflows,
  undoManager,
  deploymentVersion,
}: {
  yDoc: Doc | null;
  yWorkflows: YMap<YWorkflow>;
  undoManager: UndoManager | null;
  deploymentVersion?: string;
}) => {
  const yMetadata = useMemo(
    () => yDoc?.getMap<YDocMetadataValue>("metadata"),
    [yDoc],
  );
  const metadata = useY(yMetadata ?? emptyMetadata);
  const tracking = isDeploymentTracking(metadata?.[TRACKING_KEY])
    ? metadata[TRACKING_KEY]
    : undefined;
  const isChanged = !!tracking && metadata?.[CHANGED_KEY] === tracking.version;

  // A record for another version means the project was deployed from outside
  // the editor, so there is nothing reliable to say.
  const deploymentChangeStatus: DeploymentChangeStatus | undefined =
    tracking && deploymentVersion && tracking.version === deploymentVersion
      ? isChanged
        ? "changed"
        : "unchanged"
      : undefined;

  // The record, but only while it reads unchanged.
  const readCleanTracking = useCallback(() => {
    const value = yMetadata?.get(TRACKING_KEY);
    if (!isDeploymentTracking(value)) return undefined;
    return yMetadata?.get(CHANGED_KEY) === value.version ? undefined : value;
  }, [yMetadata]);

  const setChanged = useCallback(
    (current: DeploymentTracking, changed: boolean) => {
      const next = changed ? current.version : null;
      if (yMetadata?.get(CHANGED_KEY) === next) return;
      yMetadata?.set(CHANGED_KEY, next);
    },
    [yMetadata],
  );

  // Flags the record if the graph no longer matches the deployed fingerprint.
  const reconcile = useCallback(
    (current: DeploymentTracking) => {
      if (computeDeploymentFingerprint(yWorkflows) !== current.fingerprint) {
        setChanged(current, true);
      }
    },
    [yWorkflows, setChanged],
  );

  // Note and batch node IDs, needed to recognise their deletion. Only kept
  // while the record reads unchanged; dropped once it latches.
  const ignoredNodeIdsRef = useRef<Set<string> | null>(null);
  const isClean = !!tracking && !isChanged;

  useEffect(() => {
    ignoredNodeIdsRef.current = isClean
      ? collectIgnoredNodeIds(yWorkflows)
      : null;
  }, [isClean, yWorkflows]);

  useEffect(() => {
    const handleGraphChange = (
      events: YEvent<any>[],
      transaction: Transaction,
    ) => {
      const current = readCleanTracking();
      if (!current) return;

      // Undo/redo is compared in full below, which also rebuilds the set.
      if (undoManager && transaction.origin === undoManager) return;

      const ignored = (ignoredNodeIdsRef.current ??=
        collectIgnoredNodeIds(yWorkflows));
      const relevant = events.some((e) =>
        isDeploymentRelevantEvent(e, yWorkflows, ignored),
      );
      if (!relevant) return;

      if (transaction.local) {
        setChanged(current, true);
      } else {
        // The collaborator who made this edit flags it too, but that write can
        // lose to a concurrent deploy, so check rather than trust it.
        reconcile(current);
      }
    };

    yWorkflows.observeDeep(handleGraphChange);
    return () => yWorkflows.unobserveDeep(handleGraphChange);
  }, [yWorkflows, undoManager, readCleanTracking, setChanged, reconcile]);

  // A deploy record from another client may have been built without edits
  // this client had not yet synced.
  useEffect(() => {
    if (!yMetadata) return;

    const handleMetadataChange = (
      event: YMapEvent<YDocMetadataValue>,
      transaction: Transaction,
    ) => {
      if (transaction.local || !event.keysChanged.has(TRACKING_KEY)) return;
      const current = readCleanTracking();
      if (current) reconcile(current);
    };

    yMetadata.observe(handleMetadataChange);
    return () => yMetadata.unobserve(handleMetadataChange);
  }, [yMetadata, readCleanTracking, reconcile]);

  useEffect(() => {
    if (!undoManager) return;

    const handleUndoRedo = () => {
      const value = yMetadata?.get(TRACKING_KEY);
      if (!isDeploymentTracking(value)) return;
      const changed =
        computeDeploymentFingerprint(yWorkflows) !== value.fingerprint;
      setChanged(value, changed);
      // The graph observer skips undo/redo, so a note or batch it restored is
      // missing from the set. Rebuild it while the state stays unchanged.
      if (!changed)
        ignoredNodeIdsRef.current = collectIgnoredNodeIds(yWorkflows);
    };

    undoManager.on("stack-item-popped", handleUndoRedo);
    return () => undoManager.off("stack-item-popped", handleUndoRedo);
  }, [undoManager, yWorkflows, yMetadata, setChanged]);

  // Taken just before the workflow is built for deploying, so edits made while
  // the request is in flight still count as changes.
  const captureDeploymentFingerprint = useCallback(
    () => computeDeploymentFingerprint(yWorkflows),
    [yWorkflows],
  );

  const recordDeployment = useCallback(
    (version: string, fingerprint: string) => {
      const record: DeploymentTracking = { version, fingerprint };
      yMetadata?.set(TRACKING_KEY, record);
      if (computeDeploymentFingerprint(yWorkflows) !== fingerprint) {
        setChanged(record, true);
      }
    },
    [yMetadata, yWorkflows, setChanged],
  );

  return {
    deploymentChangeStatus,
    captureDeploymentFingerprint,
    recordDeployment,
  };
};
