import { useMemo, useRef } from "react";

import { ECHO_KEYS } from "@flow/lib/yjs";
import type { AwarenessUser, Workflow } from "@flow/types";

/**
 * What a collaborator is currently doing, resolved into names the follower can
 * read. Awareness carries ids only, so node and workflow names are looked up
 * locally against the shared doc.
 */
export type UserActivity =
  | { kind: "following"; label: string }
  | {
      kind: "subEditor";
      editor: "value" | "python" | "flowExpr";
      label: string;
    }
  | { kind: "editingNode"; label: string }
  | { kind: "workflowVariables" }
  | { kind: "assets" }
  | { kind: "nodePicker" }
  | { kind: "version"; snapshotNumber?: number }
  | { kind: "layout" }
  | { kind: "deploy" }
  | { kind: "share" }
  | { kind: "debugVariables" }
  | { kind: "debugRun"; status?: string }
  | { kind: "viewing"; label?: string };

const nodeLabel = (workflows: Workflow[], nodeId: string) => {
  for (const workflow of workflows) {
    const node = workflow.nodes?.find((n) => n.id === nodeId);
    if (node)
      return (
        node.data?.customizations?.customName || node.data?.officialName || ""
      );
  }
  return "";
};

/**
 * Most-specific-wins, except that following outranks everything: whatever a
 * follower has on screen is a consequence of who they are following, so that is
 * the more useful thing to report.
 */
const resolveActivity = (
  user: AwarenessUser,
  workflows: Workflow[],
  userNames: Record<number, string>,
): UserActivity => {
  if (user.followingClientId != null) {
    const label = userNames[user.followingClientId];
    // The followed user may have just disconnected; fall through rather than
    // showing "Following undefined".
    if (label) return { kind: "following", label };
  }

  if (user.openSubEditor) {
    const { kind, fieldId, fieldName } = user.openSubEditor;
    return {
      kind: "subEditor",
      editor: kind,
      label: fieldName || fieldId,
    };
  }

  if (user.openNodeId) {
    return {
      kind: "editingNode",
      label: nodeLabel(workflows, user.openNodeId),
    };
  }

  if (user.openNodePicker) return { kind: "nodePicker" };

  // Staging values for a debug run is a dialog like any other, but its open
  // state rides the echo channel rather than `openDialog`.
  if (user.activeElements?.includes(ECHO_KEYS.debugVariablesDialog))
    return { kind: "debugVariables" };

  switch (user.openDialog) {
    case "workflowVariables":
      return { kind: "workflowVariables" };
    case "assets":
      return { kind: "assets" };
    case "version":
      return {
        kind: "version",
        snapshotNumber: user.selectedVersionSnapshot ?? undefined,
      };
    case "layout":
      return { kind: "layout" };
    case "deploy":
      return { kind: "deploy" };
    case "share":
      return { kind: "share" };
    default:
      break;
  }

  // Legacy field, still set by the workflow variables dialog itself.
  if (user.openWorkflowVariablesDialog) return { kind: "workflowVariables" };

  if (user.debugRun?.jobId)
    return { kind: "debugRun", status: user.debugRun.status };

  const workflow = workflows.find((w) => w.id === user.currentWorkflowId);
  return { kind: "viewing", label: workflow?.name };
};

/**
 * Digest of only the fields an activity is derived from. `users` is rebuilt on
 * every awareness tick — including cursor moves — so without this the node-name
 * lookup would rescan every workflow several times a second.
 */
const activityKey = (users: AwarenessUser[]) =>
  users
    .map(
      (u) =>
        `${u.clientId}:${u.openSubEditor?.kind ?? ""}.${u.openSubEditor?.fieldId ?? ""}:${u.openNodeId ?? ""}:${u.openNodePicker ? "1" : ""}:${u.openDialog ?? ""}:${u.selectedVersionSnapshot ?? ""}:${u.openWorkflowVariablesDialog ? "1" : ""}:${u.activeElements?.join(",") ?? ""}:${u.followingClientId ?? ""}:${u.debugRun?.jobId ?? ""}.${u.debugRun?.status ?? ""}:${u.currentWorkflowId ?? ""}:${u.userName}`,
    )
    .join("|");

/**
 * Activities keyed by clientId, covering every peer plus the local user — you
 * can be following someone, and your own row should say so.
 */
export default ({
  self,
  users,
  rawWorkflows,
}: {
  self: AwarenessUser;
  users: Record<string, AwarenessUser>;
  rawWorkflows: Workflow[];
}): Record<string, UserActivity> => {
  const all = useMemo(() => [...Object.values(users), self], [users, self]);
  const key = activityKey(all);
  const allRef = useRef(all);
  allRef.current = all;

  return useMemo(() => {
    const userNames: Record<number, string> = {};
    allRef.current.forEach((user) => {
      if (user.clientId != null) userNames[user.clientId] = user.userName;
    });

    const activities: Record<string, UserActivity> = {};
    allRef.current.forEach((user) => {
      if (user.clientId == null) return;
      activities[String(user.clientId)] = resolveActivity(
        user,
        rawWorkflows,
        userNames,
      );
    });
    return activities;
    // The user list is read through a ref and represented by `key`.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key, rawWorkflows]);
};
