import { renderHook } from "@testing-library/react";
import { describe, it, expect } from "vitest";

import type { AwarenessUser, Workflow } from "@flow/types";

import useUserActivities from "./useUserActivities";

const workflows = [
  {
    id: "wf-1",
    name: "Main",
    nodes: [
      {
        id: "node-1",
        type: "transformer",
        position: { x: 0, y: 0 },
        data: { officialName: "Reprojector" },
      },
      {
        id: "node-2",
        type: "reader",
        position: { x: 0, y: 0 },
        data: {
          officialName: "FeatureReader",
          customizations: { customName: "Buildings in" },
        },
      },
    ],
  },
] as unknown as Workflow[];

const user = (overrides: Partial<AwarenessUser>): AwarenessUser => ({
  clientId: 2,
  userName: "Ada",
  color: "#fff",
  ...overrides,
});

const selfUser = user({ clientId: 1, userName: "Grace" });

const activityFor = (overrides: Partial<AwarenessUser>) => {
  const { result } = renderHook(() =>
    useUserActivities({
      self: selfUser,
      users: { "2": user(overrides) },
      rawWorkflows: workflows,
    }),
  );
  return result.current["2"];
};

describe("useUserActivities", () => {
  it("names the node being edited", () => {
    expect(activityFor({ openNodeId: "node-1" })).toEqual({
      kind: "editingNode",
      label: "Reprojector",
    });
  });

  it("prefers a node's custom name", () => {
    expect(activityFor({ openNodeId: "node-2" })).toEqual({
      kind: "editingNode",
      label: "Buildings in",
    });
  });

  it("falls back to a generic label for an unknown node", () => {
    expect(activityFor({ openNodeId: "gone" })).toEqual({
      kind: "editingNode",
      label: "",
    });
  });

  it("reports the innermost surface, not the params dialog under it", () => {
    expect(
      activityFor({
        openNodeId: "node-1",
        openSubEditor: {
          kind: "flowExpr",
          fieldId: "root_expr",
          fieldName: "expr",
        },
      }),
    ).toEqual({ kind: "subEditor", editor: "flowExpr", label: "expr" });
  });

  it("reports dialogs", () => {
    expect(activityFor({ openDialog: "workflowVariables" })).toEqual({
      kind: "workflowVariables",
    });
    expect(activityFor({ openDialog: "assets" })).toEqual({ kind: "assets" });
  });

  it("still reports the legacy workflow variables flag", () => {
    expect(activityFor({ openWorkflowVariablesDialog: true })).toEqual({
      kind: "workflowVariables",
    });
  });

  it("reports who a user is following, outranking what that opened", () => {
    const { result } = renderHook(() =>
      useUserActivities({
        self: selfUser,
        users: {
          "2": user({ clientId: 2, userName: "Ada", followingClientId: 3 }),
          "3": user({ clientId: 3, userName: "Linus", openNodeId: "node-1" }),
        },
        rawWorkflows: workflows,
      }),
    );
    expect(result.current["2"]).toEqual({ kind: "following", label: "Linus" });
    expect(result.current["3"]).toEqual({
      kind: "editingNode",
      label: "Reprojector",
    });
  });

  it("labels your own row when you are the one following", () => {
    const { result } = renderHook(() =>
      useUserActivities({
        self: user({ clientId: 1, userName: "Grace", followingClientId: 2 }),
        users: { "2": user({ clientId: 2, userName: "Ada" }) },
        rawWorkflows: workflows,
      }),
    );
    expect(result.current["1"]).toEqual({ kind: "following", label: "Ada" });
  });

  it("resolves a follow that targets the local user", () => {
    const { result } = renderHook(() =>
      useUserActivities({
        self: user({ clientId: 1, userName: "Grace" }),
        users: {
          "2": user({ clientId: 2, userName: "Ada", followingClientId: 1 }),
        },
        rawWorkflows: workflows,
      }),
    );
    expect(result.current["2"]).toEqual({ kind: "following", label: "Grace" });
  });

  it("falls through when the followed user has disconnected", () => {
    expect(
      activityFor({ followingClientId: 99, openNodeId: "node-1" }),
    ).toEqual({ kind: "editingNode", label: "Reprojector" });
  });

  it("carries the selected snapshot on the version activity", () => {
    expect(
      activityFor({ openDialog: "version", selectedVersionSnapshot: 12 }),
    ).toEqual({ kind: "version", snapshotNumber: 12 });
    expect(activityFor({ openDialog: "version" })).toEqual({
      kind: "version",
      snapshotNumber: undefined,
    });
  });

  it("reports a running debug job", () => {
    expect(
      activityFor({
        debugRun: {
          jobId: "j1",
          projectId: "p1",
          startedAt: 0,
          status: "running",
        },
      }),
    ).toEqual({ kind: "debugRun", status: "running" });
  });

  it("prefers an open dialog over a background debug run", () => {
    expect(
      activityFor({
        openDialog: "assets",
        debugRun: {
          jobId: "j1",
          projectId: "p1",
          startedAt: 0,
          status: "running",
        },
      }),
    ).toEqual({ kind: "assets" });
  });

  it("falls back to the workflow being viewed", () => {
    expect(activityFor({ currentWorkflowId: "wf-1" })).toEqual({
      kind: "viewing",
      label: "Main",
    });
  });

  it("keeps a stable result across awareness ticks that change nothing", () => {
    const { result, rerender } = renderHook(
      (props: { users: Record<string, AwarenessUser> }) =>
        useUserActivities({
          self: selfUser,
          users: props.users,
          rawWorkflows: workflows,
        }),
      { initialProps: { users: { "2": user({ openNodeId: "node-1" }) } } },
    );
    const first = result.current;

    // A cursor move rebuilds `users` without changing any activity field.
    rerender({
      users: { "2": user({ openNodeId: "node-1", cursor: { x: 9, y: 9 } }) },
    });
    expect(result.current).toBe(first);
  });
});
