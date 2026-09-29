import { act, renderHook } from "@testing-library/react";
import * as Y from "yjs";

import type { YWorkflow } from "@flow/lib/yjs/types";

import useDeploymentChanges from "./useDeploymentChanges";

const makeNode = (type = "transformer", params: unknown = {}) => {
  const yNode = new Y.Map<unknown>();
  yNode.set("type", new Y.Text(type));
  const position = new Y.Map<number>();
  position.set("x", 0);
  yNode.set("position", position);
  const data = new Y.Map<unknown>();
  data.set("params", params);
  yNode.set("data", data);
  return yNode;
};

const setup = (deploymentVersion = "v1") => {
  const doc = new Y.Doc();
  const yWorkflows = doc.getMap<YWorkflow>("workflows");
  const yWorkflow = new Y.Map<any>();
  yWorkflow.set("nodes", new Y.Map());
  yWorkflow.set("edges", new Y.Map());
  yWorkflows.set("main", yWorkflow);
  const yNodes = yWorkflow.get("nodes") as Y.Map<Y.Map<unknown>>;
  yNodes.set("a", makeNode());

  // Tracks only the local user's edits, as the editor's UndoManager does.
  const undoManager = new Y.UndoManager(yWorkflows, {
    trackedOrigins: new Set([doc.clientID]),
    captureTimeout: 0,
  });
  const edit = (fn: () => void) =>
    act(() => {
      doc.transact(fn, doc.clientID);
      undoManager.stopCapturing();
    });

  const hook = renderHook(
    ({ version }) =>
      useDeploymentChanges({
        yDoc: doc,
        yWorkflows,
        undoManager,
        deploymentVersion: version,
      }),
    { initialProps: { version: deploymentVersion } },
  );

  const deploy = (version = deploymentVersion) =>
    act(() => {
      const fingerprint = hook.result.current.captureDeploymentFingerprint();
      hook.result.current.recordDeployment(version, fingerprint);
    });

  const data = (id: string) => yNodes.get(id)?.get("data") as Y.Map<unknown>;

  return { doc, yWorkflows, yNodes, undoManager, hook, edit, deploy, data };
};

describe("useDeploymentChanges", () => {
  test("says nothing until the editor has recorded a deployment", () => {
    const { hook } = setup();
    expect(hook.result.current.deploymentChangeStatus).toBeUndefined();
  });

  test("says nothing when the record is for another version", () => {
    const { hook, deploy } = setup();
    deploy("v1");
    hook.rerender({ version: "v2" });
    expect(hook.result.current.deploymentChangeStatus).toBeUndefined();
  });

  test("marks the first relevant change, then stops writing", () => {
    const { doc, hook, edit, deploy, data } = setup();
    deploy();
    expect(hook.result.current.deploymentChangeStatus).toBe("unchanged");

    const writes = vi.fn();
    doc.getMap("metadata").observe(writes);

    edit(() => data("a").set("params", { v: 1 }));
    expect(hook.result.current.deploymentChangeStatus).toBe("changed");

    edit(() => data("a").set("params", { v: 2 }));
    edit(() => data("a").set("params", { v: 3 }));
    expect(writes).toHaveBeenCalledTimes(1);
  });

  test("ignores positions, notes and batches", () => {
    const { yNodes, hook, edit, deploy } = setup();
    deploy();

    edit(() =>
      (yNodes.get("a")?.get("position") as Y.Map<number>).set("x", 40),
    );
    edit(() => yNodes.set("n", makeNode("note")));
    edit(() => yNodes.set("b", makeNode("batch")));
    edit(() => yNodes.delete("n"));

    expect(hook.result.current.deploymentChangeStatus).toBe("unchanged");
  });

  test("counts deleting an action node", () => {
    const { yNodes, hook, edit, deploy } = setup();
    deploy();
    edit(() => yNodes.delete("a"));
    expect(hook.result.current.deploymentChangeStatus).toBe("changed");
  });

  test("undo back to the deployed state clears it, and redo sets it", () => {
    const { undoManager, hook, edit, deploy, data } = setup();
    deploy();

    edit(() => data("a").set("params", { v: 1 }));
    expect(hook.result.current.deploymentChangeStatus).toBe("changed");

    act(() => {
      undoManager.undo();
    });
    expect(hook.result.current.deploymentChangeStatus).toBe("unchanged");

    act(() => {
      undoManager.redo();
    });
    expect(hook.result.current.deploymentChangeStatus).toBe("changed");
  });

  test("undo of an unrelated edit leaves it changed", () => {
    const { yNodes, undoManager, hook, edit, deploy, data } = setup();
    deploy();

    edit(() => data("a").set("params", { v: 1 }));
    edit(() =>
      (yNodes.get("a")?.get("position") as Y.Map<number>).set("x", 40),
    );
    act(() => {
      undoManager.undo();
    });

    expect(hook.result.current.deploymentChangeStatus).toBe("changed");
  });

  test("leaves writing the flag to the collaborator who made the edit", () => {
    const { doc, hook, deploy } = setup();
    deploy();

    // A second client edits and its update arrives here as a remote change.
    const remote = new Y.Doc();
    Y.applyUpdate(remote, Y.encodeStateAsUpdate(doc));
    const remoteData = (
      remote.getMap<YWorkflow>("workflows").get("main")?.get("nodes") as Y.Map<
        Y.Map<unknown>
      >
    )
      .get("a")
      ?.get("data") as Y.Map<unknown>;
    remote.transact(() => remoteData.set("params", { v: 9 }));

    act(() => {
      Y.applyUpdate(
        doc,
        Y.encodeStateAsUpdate(remote, Y.encodeStateVector(doc)),
      );
    });
    expect(hook.result.current.deploymentChangeStatus).toBe("unchanged");
  });

  test("an edit made while the deploy is in flight still counts", () => {
    const { hook, edit, data } = setup();
    const fingerprint = hook.result.current.captureDeploymentFingerprint();
    edit(() => data("a").set("params", { v: 1 }));
    act(() => hook.result.current.recordDeployment("v1", fingerprint));
    expect(hook.result.current.deploymentChangeStatus).toBe("changed");
  });
});
