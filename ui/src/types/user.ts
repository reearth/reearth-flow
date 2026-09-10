import { ApiResponse } from "./api";
import { JobStatus } from "./job";
import type { Workspace } from "./workspace";

export type Me = {
  myWorkspaceId: string;
  lang?: string;
  theme?: string;
} & User;

export type GetMe = {
  me: Me | undefined;
  isLoading: boolean;
} & ApiResponse;

export type GetMeAndWorkspaces = {
  me: Me | undefined;
  workspaces: Workspace[] | undefined;
  isLoading: boolean;
} & ApiResponse;

export type User = {
  id: string;
  name: string;
  email: string;
};

export type SearchUser = {
  user?: User;
};

export type UpdateMe = {
  me?: User;
} & ApiResponse;

export type AwarenessSelection = { color: string; userName: string };
export type AwarenessSelectionsMap = Record<string, AwarenessSelection[]>;
/** Users currently on each echoed UI element, keyed by that element's key. */
export type AwarenessEchoMap = Record<string, AwarenessSelection[]>;

/**
 * Dialogs whose open/closed state is broadcast over awareness. Mirrors the
 * editor's local `DialogOptions`, kept as its own union so `@flow/types` does
 * not have to reach into a feature directory.
 */
export type AwarenessDialog =
  | "deploy"
  | "share"
  | "version"
  | "assets"
  | "debugStop"
  | "workflowVariables"
  | "collaboration"
  | "layout";

/** A code/value editor opened on top of the params dialog. */
export type AwarenessSubEditor = {
  kind: "value" | "python" | "flowExpr";
  /** RJSF field id (e.g. "root_expr"), used to reopen the same field. */
  fieldId: string;
  fieldName: string;
};

export type AwarenessNodePicker = {
  nodeType: string;
  position: { x: number; y: number };
};

export type AwarenessUser = {
  clientId: number;
  selectionRect?: {
    startX: number;
    startY: number;
    currentX: number;
    currentY: number;
  } | null;
  cursor?: {
    x: number;
    y: number;
  };
  viewport?: {
    x: number;
    y: number;
    zoom: number;
  };
  focusedElement?: boolean;
  openNodeId?: string | null;
  focusedParamField?: string | null;
  openSubEditor?: AwarenessSubEditor | null;
  openDialog?: AwarenessDialog | null;
  openNodePicker?: AwarenessNodePicker | null;
  openWorkflowVariablesDialog?: boolean | null;
  /** Snapshot selected in the version history dialog (preview, not restore). */
  selectedVersionSnapshot?: number | null;
  /** clientId of the user this user is spotlighting, if any. */
  followingClientId?: number | null;
  /**
   * Stable keys for the UI elements this user is currently on — a dropdown they
   * have open, a list row they are hovering, a menu item under their pointer.
   * Read with `useAwarenessEcho(key)`; any component can opt in with one line,
   * so this stays a single field rather than growing one per surface.
   */
  activeElements?: string[] | null;
  focusedVariableId?: string | null;
  focusedVariableField?: string | null;
  editingVariableId?: string | null;
  color: string;
  userName: string;
  currentWorkflowId?: string;
  openWorkflowIds?: string[];
  debugRun?: UserDebugRun;
  draggingEdge?: {
    nodeId: string;
    handleId: string | null;
    handleType: "source" | "target" | null;
  } | null;
  selectedNodeIds?: string[] | null;
};

export type UserDebugRun = {
  projectId: string;
  jobId: string;
  startedAt: number;
  status: JobStatus;
};
