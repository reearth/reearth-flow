import { act, renderHook } from "@testing-library/react";
import * as Y from "yjs";

import { computeDeploymentFingerprint } from "@flow/lib/yjs/deploymentTracking";
import type { YWorkflow } from "@flow/lib/yjs/types";

import useDeploymentChanges from "./useDeploymentChanges";

// Wrapped so tests can count how often the graph is fingerprinted.
vi.mock("@flow/lib/yjs/deploymentTracking", async (importOriginal) => {
  const actual =
    await importOriginal<typeof import("@flow/lib/yjs/deploymentTracking")>();
  return {
    ...actual,
    computeDeploymentFingerprint: vi.fn(actual.computeDeploymentFingerprint),
  };
});

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

  test("a note restored by undo is still recognised when deleted again", () => {
    const { yNodes, undoManager, hook, edit, deploy } = setup();
    edit(() => yNodes.set("n", makeNode("note")));
    deploy();

    edit(() => yNodes.delete("n"));
    act(() => {
      undoManager.undo();
    });
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

  test("flags a remote edit even when its author did not", () => {
    const { doc, hook, deploy } = setup();
    deploy();

    // A second client without this hook edits, so nothing flags it there, and
    // its update arrives here as a remote change.
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
    expect(hook.result.current.deploymentChangeStatus).toBe("changed");
  });

  test("an edit made while the deploy is in flight still counts", () => {
    const { hook, edit, data } = setup();
    const fingerprint = hook.result.current.captureDeploymentFingerprint();
    edit(() => data("a").set("params", { v: 1 }));
    act(() => hook.result.current.recordDeployment("v1", fingerprint));
    expect(hook.result.current.deploymentChangeStatus).toBe("changed");
  });
});

// Two collaborators on separate docs, synced by hand. Yjs settles concurrent
// writes to one map key by client ID, so the tests run both ways round.
const createClient = (clientID: number, from?: Y.Doc) => {
  const doc = new Y.Doc();
  doc.clientID = clientID;
  const yWorkflows = doc.getMap<YWorkflow>("workflows");
  if (from) {
    Y.applyUpdate(doc, Y.encodeStateAsUpdate(from));
  } else {
    const yWorkflow = new Y.Map<any>();
    yWorkflow.set("nodes", new Y.Map());
    yWorkflow.set("edges", new Y.Map());
    yWorkflows.set("main", yWorkflow);
    (yWorkflow.get("nodes") as Y.Map<Y.Map<unknown>>).set("a", makeNode());
  }
  const undoManager = new Y.UndoManager(yWorkflows, {
    trackedOrigins: new Set([doc.clientID]),
  });
  const hook = renderHook(
    ({ version }) =>
      useDeploymentChanges({
        yDoc: doc,
        yWorkflows,
        undoManager,
        deploymentVersion: version,
      }),
    { initialProps: { version: "v3" } },
  );
  const params = () =>
    (
      (yWorkflows.get("main")?.get("nodes") as Y.Map<Y.Map<unknown>>)
        .get("a")
        ?.get("data") as Y.Map<unknown>
    ).get("params");
  return {
    doc,
    hook,
    status: () => hook.result.current.deploymentChangeStatus,
    setParams: (value: unknown) =>
      act(() => {
        doc.transact(() => {
          (
            (yWorkflows.get("main")?.get("nodes") as Y.Map<Y.Map<unknown>>)
              .get("a")
              ?.get("data") as Y.Map<unknown>
          ).set("params", value);
        }, doc.clientID);
      }),
    params,
    deploy: (version: string) =>
      act(() => {
        const fp = hook.result.current.captureDeploymentFingerprint();
        hook.result.current.recordDeployment(version, fp);
      }),
    setVersion: (version: string) => hook.rerender({ version }),
  };
};

type Client = ReturnType<typeof createClient>;

const sync = (a: Client, b: Client) =>
  act(() => {
    const toB = Y.encodeStateAsUpdate(a.doc, Y.encodeStateVector(b.doc));
    const toA = Y.encodeStateAsUpdate(b.doc, Y.encodeStateVector(a.doc));
    Y.applyUpdate(b.doc, toB);
    Y.applyUpdate(a.doc, toA);
  });

describe("useDeploymentChanges with collaborators", () => {
  test.each([
    [1, 2],
    [2, 1],
  ])(
    "an edit made while another client deploys is flagged (ids %i, %i)",
    (deployerId, editorId) => {
      const deployer = createClient(deployerId);
      deployer.deploy("v3");
      const editor = createClient(editorId, deployer.doc);
      expect(editor.status()).toBe("unchanged");

      // Neither client sees the other's change before the deploy lands.
      editor.setParams({ v: 1 });
      deployer.deploy("v4");
      deployer.setVersion("v4");
      editor.setVersion("v4");
      sync(deployer, editor);

      expect(deployer.status()).toBe("changed");
      expect(editor.status()).toBe("changed");
    },
  );

  test("catching up on edits that were deployed does not flag them", () => {
    const deployer = createClient(1);
    deployer.deploy("v3");
    const viewer = createClient(2, deployer.doc);

    // The viewer is offline while the deployer edits and redeploys.
    deployer.setParams({ v: 1 });
    deployer.deploy("v4");
    deployer.setVersion("v4");
    viewer.setVersion("v4");
    sync(deployer, viewer);

    expect(viewer.params()).toEqual({ v: 1 });
    expect(viewer.status()).toBe("unchanged");
    expect(deployer.status()).toBe("unchanged");
  });
});

test("once flagged, remote edits are not fingerprinted", () => {
  const deployer = createClient(1);
  deployer.deploy("v3");
  const editor = createClient(2, deployer.doc);
  editor.setParams({ v: 1 });
  sync(deployer, editor);
  expect(deployer.status()).toBe("changed");

  vi.mocked(computeDeploymentFingerprint).mockClear();
  for (let v = 2; v < 5; v++) {
    editor.setParams({ v });
    sync(deployer, editor);
  }
  expect(computeDeploymentFingerprint).not.toHaveBeenCalled();
});
