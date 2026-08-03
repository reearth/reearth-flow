import { useDocument } from "@flow/lib/gql/document/useApi";

export default ({ projectId }: { projectId: string }) => {
  const { useGetProjectSnapshots, useGetLatestProjectSnapshot } = useDocument();
  // Version history is snapshot-backed. The raw CRDT update log is a
  // durability concern and is deliberately not shown: it has one entry per
  // flush.
  const { snapshots, isFetching } = useGetProjectSnapshots(projectId);
  const { projectDocument } = useGetLatestProjectSnapshot(projectId);
  const latestProjectSnapshotVersion = projectDocument;

  // NamedSnapshot.id and the raw update-log `version` consumed by
  // previewSnapshot/rollbackProject are distinct, backend-assigned ID
  // spaces (see server/websocket-go/internal/gcs/snapshots.go,
  // SnapNextIDName, vs. the update-log clock read by
  // GetHistoryByVersion/Rollback in
  // server/api/internal/usecase/interactor/websocket.go). There is no
  // client-side translation between them. Previewing or reverting to a
  // NamedSnapshot needs a GraphQL query keyed by snapshot id
  // (previewNamedSnapshot(projectId, snapshotId), backed by the existing
  // GET /api/document/{id}/snapshots/{sid} endpoint) that does not exist
  // yet; that work is tracked separately. Until it lands, snapshot rows
  // are read-only: this panel intentionally has no preview-on-click and no
  // revert action.

  return {
    snapshots,
    latestProjectSnapshotVersion,
    isFetching,
  };
};
