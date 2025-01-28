import { renderHook } from "@testing-library/react";
import * as Y from "yjs";

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
      { id: "main", name: "Workflow-1" },
    ]);
  });
});
