import { useDocument } from "@flow/lib/gql/document/useApi";

// This panel has two distinct consumers with two distinct data sources,
// and they must not be merged:
//
// - Normal browsing (this hook, used by ./index.tsx) wants user-meaningful
//   named versions: the snapshot list. NamedSnapshot.id and the raw
//   update-log `version` consumed by previewSnapshot/rollbackProject are
//   different, backend-assigned ID spaces (see
//   server/websocket-go/internal/gcs/snapshots.go, SnapNextIDName, vs. the
//   update-log clock read by GetHistoryByVersion/Rollback in
//   server/api/internal/usecase/interactor/websocket.go) with no correct
//   client-side translation between them. Feeding a snapshot id into
//   rollbackProject rebuilds the project at an unrelated update-log clock
//   and durably prunes every update after it — real, irrecoverable data
//   loss, not just a wrong preview. So this hook intentionally exposes no
//   preview-on-click and no revert action; the panel is read-only.
// - Project-corruption recovery (see ./recoveryHooks.ts, used by
//   ./RecoveryDialog.tsx) wants "get me back to any working state" for a
//   project that will not open at all, and stays on the raw update log
//   end to end: `projectHistory` entries carry the same `version` that
//   previewSnapshot/rollbackProject expect, so that mapping is correct and
//   safe. Do not read the paragraph above as implying recovery mode is
//   also unsafe — it uses a completely different data source that was
//   never affected by this ID-space mismatch.
//
// A GraphQL query keyed by snapshot id (previewNamedSnapshot(projectId,
// snapshotId), backed by the existing GET /api/document/{id}/snapshots/{sid}
// endpoint) would let normal-mode preview/revert work correctly; that is
// tracked separately and not yet implemented.
export default ({ projectId }: { projectId: string }) => {
  const { useGetProjectSnapshots, useGetLatestProjectSnapshot } = useDocument();
  const { snapshots, isFetching } = useGetProjectSnapshots(projectId);
  const { projectDocument } = useGetLatestProjectSnapshot(projectId);
  const latestProjectSnapshotVersion = projectDocument;

  return {
    snapshots,
    latestProjectSnapshotVersion,
    isFetching,
  };
};
