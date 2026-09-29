import { useCallback, useEffect, useMemo, useRef } from "react";
import { useY } from "react-yjs";
import { Map as YMap } from "yjs";
import type { Doc, Transaction, YEvent, UndoManager } from "yjs";

import {
  collectIgnoredNodeIds,
  computeDeploymentFingerprint,
  isDeploymentRelevantEvent,
} from "@flow/lib/yjs/deploymentTracking";
import type { YWorkflow } from "@flow/lib/yjs/types";
import type { DeploymentTracking, YDocMetadataValue } from "@flow/types";

const emptyMetadata = new YMap<YDocMetadataValue>();

const TRACKING_KEY = "deploymentTracking";

export type DeploymentChangeStatus = "changed" | "unchanged";

const isDeploymentTracking = (value: unknown): value is DeploymentTracking =>
  typeof value === "object" &&
  value !== null &&
  "version" in value &&
  "fingerprint" in value &&
  "changed" in value;

// Tracks whether the canvas has changed since the project was last deployed
// from the editor. The record lives in the Yjs doc's metadata map, so every
// collaborator shares it, it persists with the project, and the UndoManager
// (scoped to the workflows map) never touches it.
//
// It is a latch: the first relevant edit marks it changed and every later edit
// returns after one map read. Only undo/redo, which can restore the deployed
// state, compares the graph fingerprint again.
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

  // A record for another version means the project was deployed from outside
  // the editor, so there is nothing reliable to say.
  const deploymentChangeStatus: DeploymentChangeStatus | undefined =
    tracking && deploymentVersion && tracking.version === deploymentVersion
      ? tracking.changed
        ? "changed"
        : "unchanged"
      : undefined;

  const readTracking = useCallback(() => {
    const value = yMetadata?.get(TRACKING_KEY);
    return isDeploymentTracking(value) ? value : undefined;
  }, [yMetadata]);

  const writeChanged = useCallback(
    (current: DeploymentTracking, changed: boolean) => {
      if (current.changed === changed) return;
      yMetadata?.set(TRACKING_KEY, { ...current, changed });
    },
    [yMetadata],
  );

  // Note and batch node IDs, needed to recognise their deletion. Only kept
  // while the record is unchanged; dropped once it latches.
  const ignoredNodeIdsRef = useRef<Set<string> | null>(null);
  const isClean = !!tracking && !tracking.changed;

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
      const current = readTracking();
      if (!current || current.changed) return;

      // Undo/redo is compared in full below.
      if (undoManager && transaction.origin === undoManager) return;

      const ignored = (ignoredNodeIdsRef.current ??=
        collectIgnoredNodeIds(yWorkflows));
      const relevant = events.some((e) =>
        isDeploymentRelevantEvent(e, yWorkflows, ignored),
      );

      // Every client keeps its ignored-node set current, but only the one
      // that made the edit writes the flag.
      if (relevant && transaction.local) writeChanged(current, true);
    };

    yWorkflows.observeDeep(handleGraphChange);
    return () => yWorkflows.unobserveDeep(handleGraphChange);
  }, [yWorkflows, undoManager, readTracking, writeChanged]);

  useEffect(() => {
    if (!undoManager) return;

    const handleUndoRedo = () => {
      const current = readTracking();
      if (!current) return;
      writeChanged(
        current,
        computeDeploymentFingerprint(yWorkflows) !== current.fingerprint,
      );
    };

    undoManager.on("stack-item-popped", handleUndoRedo);
    return () => undoManager.off("stack-item-popped", handleUndoRedo);
  }, [undoManager, yWorkflows, readTracking, writeChanged]);

  // Taken just before the workflow is built for deploying, so edits made while
  // the request is in flight still count as changes.
  const captureDeploymentFingerprint = useCallback(
    () => computeDeploymentFingerprint(yWorkflows),
    [yWorkflows],
  );

  const recordDeployment = useCallback(
    (version: string, fingerprint: string) => {
      yMetadata?.set(TRACKING_KEY, {
        version,
        fingerprint,
        changed: computeDeploymentFingerprint(yWorkflows) !== fingerprint,
      });
    },
    [yMetadata, yWorkflows],
  );

  return {
    deploymentChangeStatus,
    captureDeploymentFingerprint,
    recordDeployment,
  };
};
