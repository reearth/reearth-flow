import { act, cleanup, renderHook } from "@testing-library/react";
import { ReactFlowProvider, type EdgeChange } from "@xyflow/react";
import { createElement } from "react";
import { afterEach, describe, expect, test } from "vitest";

import type { Edge, Node } from "@flow/types";

import useNodes from "./useNodes";

afterEach(() => cleanup());

const transformer = (id: string): Node => ({
  id,
  type: "transformer",
  position: { x: 0, y: 0 },
  data: { officialName: id, inputs: ["features"], outputs: ["features"] },
});

const edgeBetween = (id: string, source: string, target: string): Edge => ({
  id,
  source,
  target,
  sourceHandle: "features",
  targetHandle: "features",
});

const runCleanup = (nodes: Node[], edges: Edge[], deleted: Node[]) => {
  const emitted: EdgeChange[] = [];
  const { result } = renderHook(
    () =>
      useNodes({
        nodes,
        edges,
        onEdgesChange: (changes) => emitted.push(...changes),
      }),
    {
      wrapper: ({ children }) =>
        createElement(ReactFlowProvider, null, children),
    },
  );

  act(() => result.current.handleNodesDeleteCleanup(deleted));

  const removedIds = emitted
    .filter((c) => c.type === "remove")
    .map((c) => (c as { id: string }).id);
  const added = emitted
    .filter((c) => c.type === "add")
    .map((c) => (c as { item: Edge }).item);

  return {
    removedIds,
    added,
    surviving: [...edges.filter((e) => !removedIds.includes(e.id)), ...added],
  };
};

describe("handleNodesDeleteCleanup", () => {
  test("reconnects the neighbours of a deleted node", () => {
    const { surviving } = runCleanup(
      [transformer("a"), transformer("b"), transformer("c")],
      [edgeBetween("a-b", "a", "b"), edgeBetween("b-c", "b", "c")],
      [transformer("b")],
    );

    expect(surviving).toHaveLength(1);
    expect(surviving[0]).toMatchObject({ source: "a", target: "c" });
  });

  test("does not duplicate a connection that already exists", () => {
    const { surviving } = runCleanup(
      [transformer("a"), transformer("b"), transformer("c")],
      [
        edgeBetween("a-b", "a", "b"),
        edgeBetween("b-c", "b", "c"),
        edgeBetween("a-c", "a", "c"),
      ],
      [transformer("b")],
    );

    const aToC = surviving.filter((e) => e.source === "a" && e.target === "c");
    expect(aToC).toHaveLength(1);
    expect(aToC[0].id).toBe("a-c");
  });

  test("removes every edge attached to the deleted nodes", () => {
    const { removedIds } = runCleanup(
      [transformer("a"), transformer("b"), transformer("c")],
      [edgeBetween("a-b", "a", "b"), edgeBetween("b-c", "b", "c")],
      [transformer("b")],
    );

    expect(removedIds.sort()).toEqual(["a-b", "b-c"]);
  });

  test("does not reconnect through nodes that are also being deleted", () => {
    const { surviving } = runCleanup(
      [transformer("a"), transformer("b"), transformer("c")],
      [edgeBetween("a-b", "a", "b"), edgeBetween("b-c", "b", "c")],
      [transformer("b"), transformer("c")],
    );

    expect(surviving).toEqual([]);
  });
});
