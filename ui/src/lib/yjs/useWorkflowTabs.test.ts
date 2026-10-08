import { act, renderHook } from "@testing-library/react";
import { useState } from "react";
import * as Y from "yjs";

import { DEFAULT_ENTRY_GRAPH_ID as MAIN } from "@flow/global-constants";
import type { Workflow } from "@flow/types";

import { rebuildWorkflow, yWorkflowConstructor } from "./conversions";
import type { YWorkflow } from "./types";
import useWorkflowTabs from "./useWorkflowTabs";

describe("useWorkflowTabs", () => {
  const yDoc = new Y.Doc();
  const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
  const yWorkflowMain = yWorkflowConstructor("main", "Workflow-1");
  const yWorkflow2 = yWorkflowConstructor("2", "Workflow-2");
  yWorkflows.push([yWorkflowMain, yWorkflow2]);
  const currentWorkflowId = "main";

  const { result: result1 } = renderHook(() =>
    yWorkflows.map((w) => rebuildWorkflow(w)),
  );

  const { result: result2 } = renderHook(() =>
    useWorkflowTabs({
      currentWorkflowId,
      rawWorkflows: result1.current,
      setCurrentWorkflowId: vi.fn(),
    }),
  );

  it("should set isMainWorkflow to true when main is currentWorkflowId", () => {
    expect(result2.current.isMainWorkflow).toBe(true);
  });
  it("should set openWorkflows appropriately", () => {
    expect(result2.current.openWorkflows).toEqual([
      { id: "main", name: "Workflow-1", depth: 0, parentId: undefined },
    ]);
  });
});

describe("useWorkflowTabs nesting", () => {
  const workflow = (id: string, subworkflowIds: string[] = []): Workflow => ({
    id,
    name: id,
    nodes: subworkflowIds.map((subworkflowId) => ({
      id: `node-${subworkflowId}`,
      type: "subworkflow",
      position: { x: 0, y: 0 },
      data: { officialName: "Subworkflow", subworkflowId },
    })),
    edges: [],
  });

  // main → a → b → c, main → d
  const rawWorkflows = [
    workflow(MAIN, ["a", "d"]),
    workflow("a", ["b"]),
    workflow("b", ["c"]),
    workflow("c"),
    workflow("d"),
  ];

  const renderTabs = () =>
    renderHook(() => {
      const [currentWorkflowId, setCurrentWorkflowId] = useState(MAIN);
      return {
        currentWorkflowId,
        ...useWorkflowTabs({
          currentWorkflowId,
          rawWorkflows,
          setCurrentWorkflowId,
        }),
      };
    });

  const tree = (
    openWorkflows: { id: string; depth: number; parentId?: string }[],
  ) =>
    openWorkflows.map(({ id, depth, parentId }) => ({ id, depth, parentId }));

  it("opens every parent of a nested subworkflow", () => {
    const { result } = renderTabs();
    act(() => result.current.handleWorkflowOpen("c"));

    expect(result.current.currentWorkflowId).toBe("c");
    expect(tree(result.current.openWorkflows)).toEqual([
      { id: MAIN, depth: 0, parentId: undefined },
      { id: "a", depth: 1, parentId: MAIN },
      { id: "b", depth: 2, parentId: "a" },
      { id: "c", depth: 3, parentId: "b" },
    ]);
  });

  it("lists subworkflows under their parent regardless of open order", () => {
    const { result } = renderTabs();
    act(() => result.current.handleWorkflowOpen("a"));
    act(() => result.current.handleWorkflowOpen("d"));
    act(() => result.current.handleWorkflowOpen("b"));

    expect(result.current.openWorkflows.map((wf) => wf.id)).toEqual([
      MAIN,
      "a",
      "b",
      "d",
    ]);
  });

  it("closes nested subworkflows with their parent and moves up", () => {
    const { result } = renderTabs();
    act(() => result.current.handleWorkflowOpen("c"));
    act(() => result.current.handleWorkflowOpen("d"));
    act(() => result.current.handleWorkflowOpen("c"));
    act(() => result.current.handleWorkflowClose("b"));

    expect(result.current.openWorkflowIds).toEqual([MAIN, "a", "d"]);
    expect(result.current.currentWorkflowId).toBe("a");
  });

  it("stays put when closing a workflow the user is not in", () => {
    const { result } = renderTabs();
    act(() => result.current.handleWorkflowOpen("c"));
    act(() => result.current.handleWorkflowOpen("d"));
    act(() => result.current.handleWorkflowClose("a"));

    expect(result.current.openWorkflowIds).toEqual([MAIN, "d"]);
    expect(result.current.currentWorkflowId).toBe("d");
  });
});
