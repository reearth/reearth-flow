import * as Y from "yjs";

import {
  collectIgnoredNodeIds,
  computeDeploymentFingerprint,
  isDeploymentRelevantEvent,
} from "./deploymentTracking";
import type { YWorkflow } from "./types";

type NodeInit = {
  type?: string;
  params?: Record<string, unknown>;
  x?: number;
};

const makeNode = ({ type = "transformer", params = {}, x = 0 }: NodeInit) => {
  const yNode = new Y.Map<unknown>();
  yNode.set("type", new Y.Text(type));
  const position = new Y.Map<number>();
  position.set("x", x);
  position.set("y", 0);
  yNode.set("position", position);
  const data = new Y.Map<unknown>();
  data.set("officialName", new Y.Text("Action"));
  data.set("params", params);
  yNode.set("data", data);
  return yNode;
};

const makeEdge = (source: string, target: string) => {
  const yEdge = new Y.Map<Y.Text>();
  yEdge.set("source", new Y.Text(source));
  yEdge.set("target", new Y.Text(target));
  return yEdge;
};

const setup = () => {
  const doc = new Y.Doc();
  const yWorkflows = doc.getMap<YWorkflow>("workflows");
  const yWorkflow = new Y.Map<any>();
  yWorkflow.set("nodes", new Y.Map());
  yWorkflow.set("edges", new Y.Map());
  yWorkflows.set("main", yWorkflow);
  const yNodes = yWorkflow.get("nodes") as Y.Map<Y.Map<unknown>>;
  const yEdges = yWorkflow.get("edges") as Y.Map<Y.Map<Y.Text>>;
  return { doc, yWorkflows, yNodes, yEdges };
};

const data = (yNodes: Y.Map<Y.Map<unknown>>, id: string) =>
  yNodes.get(id)?.get("data") as Y.Map<unknown>;
const position = (yNodes: Y.Map<Y.Map<unknown>>, id: string) =>
  yNodes.get(id)?.get("position") as Y.Map<number>;

// Runs a change and reports whether any event it produced was relevant.
const relevance = (
  yWorkflows: Y.Map<YWorkflow>,
  ignored: Set<string>,
  change: () => void,
) => {
  let relevant = false;
  const handler = (events: Y.YEvent<any>[]) => {
    relevant = events.some((e) =>
      isDeploymentRelevantEvent(e, yWorkflows, ignored),
    );
  };
  yWorkflows.observeDeep(handler);
  change();
  yWorkflows.unobserveDeep(handler);
  return relevant;
};

describe("isDeploymentRelevantEvent", () => {
  test("counts adding and removing an action node", () => {
    const { yWorkflows, yNodes } = setup();
    const ignored = new Set<string>();
    expect(
      relevance(yWorkflows, ignored, () => yNodes.set("a", makeNode({}))),
    ).toBe(true);
    expect(relevance(yWorkflows, ignored, () => yNodes.delete("a"))).toBe(true);
  });

  test("ignores adding and removing note and batch nodes", () => {
    const { yWorkflows, yNodes } = setup();
    const ignored = new Set<string>();
    expect(
      relevance(yWorkflows, ignored, () => {
        yNodes.set("n", makeNode({ type: "note" }));
        yNodes.set("b", makeNode({ type: "batch" }));
      }),
    ).toBe(false);
    expect(ignored).toEqual(new Set(["n", "b"]));
    expect(relevance(yWorkflows, ignored, () => yNodes.delete("n"))).toBe(
      false,
    );
    expect(ignored).toEqual(new Set(["b"]));
  });

  test("counts deleting a node it has no record of", () => {
    const { yWorkflows, yNodes } = setup();
    yNodes.set("n", makeNode({ type: "note" }));
    expect(relevance(yWorkflows, new Set(), () => yNodes.delete("n"))).toBe(
      true,
    );
  });

  test("counts param and disabled changes on action nodes only", () => {
    const { yWorkflows, yNodes } = setup();
    yNodes.set("a", makeNode({}));
    yNodes.set("n", makeNode({ type: "note" }));
    const ignored = collectIgnoredNodeIds(yWorkflows);

    expect(
      relevance(yWorkflows, ignored, () =>
        data(yNodes, "a").set("params", { v: 1 }),
      ),
    ).toBe(true);
    expect(
      relevance(yWorkflows, ignored, () =>
        data(yNodes, "a").set("isDisabled", true),
      ),
    ).toBe(true);
    expect(
      relevance(yWorkflows, ignored, () =>
        data(yNodes, "n").set("params", { content: "hi" }),
      ),
    ).toBe(false);
  });

  test("ignores position and other data fields", () => {
    const { yWorkflows, yNodes } = setup();
    yNodes.set("a", makeNode({}));
    const ignored = collectIgnoredNodeIds(yWorkflows);

    expect(
      relevance(yWorkflows, ignored, () => position(yNodes, "a").set("x", 50)),
    ).toBe(false);
    expect(
      relevance(yWorkflows, ignored, () =>
        data(yNodes, "a").set("customName", "Renamed"),
      ),
    ).toBe(false);
  });

  test("counts edge changes", () => {
    const { yWorkflows, yEdges } = setup();
    const ignored = new Set<string>();
    expect(
      relevance(yWorkflows, ignored, () => yEdges.set("e", makeEdge("a", "b"))),
    ).toBe(true);
    expect(relevance(yWorkflows, ignored, () => yEdges.delete("e"))).toBe(true);
  });
});

describe("computeDeploymentFingerprint", () => {
  test("is unaffected by positions, notes and param key order", () => {
    const { yWorkflows, yNodes } = setup();
    yNodes.set("a", makeNode({ params: { x: 1, y: 2 } }));
    const before = computeDeploymentFingerprint(yWorkflows);

    position(yNodes, "a").set("x", 99);
    yNodes.set("n", makeNode({ type: "note" }));
    data(yNodes, "a").set("params", { y: 2, x: 1 });

    expect(computeDeploymentFingerprint(yWorkflows)).toBe(before);
  });

  test("changes with params, nodes and edges, and returns when reverted", () => {
    const { yWorkflows, yNodes, yEdges } = setup();
    yNodes.set("a", makeNode({ params: { v: 1 } }));
    const before = computeDeploymentFingerprint(yWorkflows);

    data(yNodes, "a").set("params", { v: 2 });
    expect(computeDeploymentFingerprint(yWorkflows)).not.toBe(before);
    data(yNodes, "a").set("params", { v: 1 });
    expect(computeDeploymentFingerprint(yWorkflows)).toBe(before);

    yEdges.set("e", makeEdge("a", "a"));
    expect(computeDeploymentFingerprint(yWorkflows)).not.toBe(before);
    yEdges.delete("e");

    yNodes.set("b", makeNode({}));
    expect(computeDeploymentFingerprint(yWorkflows)).not.toBe(before);
  });
});
