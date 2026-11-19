import type { ApiResponse } from "./api";
import type { User } from "./user";

export type ProjectSnapshotMeta = {
  timestamp: string;
  version: number;
};

// A labelled, named snapshot in a project's version history (distinct from
// ProjectSnapshot, which is a raw update-vector snapshot keyed by version).
// The label the websocket server gives a version it saves on its own (every 15
// minutes while edited, and when the last editor leaves). Rows carrying it are
// not user-named, so the UI shows "Autosaved" instead.
export const AUTO_SNAPSHOT_LABEL = "auto";
// The labels the API gives versions saved by an action: the editor's save
// (Ctrl+S), a debug run and a deployment. They are codes, shown as text of
// their own.
export const SAVE_SNAPSHOT_LABEL = "save";
export const RUN_SNAPSHOT_LABEL = "run";
export const DEPLOY_SNAPSHOT_LABEL = "deploy";

// Someone whose edits a version holds.
export type SnapshotContributor = Pick<User, "id" | "name" | "photoURL">;

export type NamedSnapshot = {
  snapshotNumber: number;
  label: string;
  timestamp: string;
  size: number;
  // The contributors that resolve to a user. Empty for versions saved without
  // attribution, and possibly shorter than contributorCount: the stored list is
  // capped, and users that no longer exist are left out.
  contributors: SnapshotContributor[];
  // How many contributors the version has, shown or not.
  contributorCount: number;
  lastEditorId?: string;
};
export type ProjectSnapshot = {
  timestamp: string;
  // Absent on a snapshot-addressed read, which has no update-log clock.
  version?: number;
  updates: number[];
};

export type ProjectDocument = {
  id: string;
  timestamp: string;
  version: number;
  updates: number[];
};

export type PreviewSnapshot = {
  id: string;
  timestamp: string;
  version: number;
  updates: number[];
};

export type RollbackProject = {
  projectDocument?: ProjectDocument;
} & ApiResponse;

export type SaveSnapshot = {
  saveSnapshot: boolean;
} & ApiResponse;
