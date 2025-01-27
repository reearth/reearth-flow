import { renderHook } from "@testing-library/react";
import * as Y from "yjs";

import { rebuildWorkflow, yWorkflowConstructor } from "./conversions";
import type { YWorkflow } from "./types";
import useWorkflowTabs from "./useWorkflowTabs";

describe("useWorkflowTabs", () => {
  it("should initialize with the first workflow as active", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
    const yWorkflow = yWorkflowConstructor("1", "Workflow-1");
    yWorkflows.push([yWorkflow]);

    const currentWorkflowId = "1";
    const { result: result1 } = renderHook(() =>
      yWorkflows.map((w) => rebuildWorkflow(w)),
    );

    const { result: result2 } = renderHook(() =>
      useWorkflowTabs({
        currentWorkflowId,
        rawWorkflows: result1.current,
        handleCurrentWorkflowIdChange: vi.fn(),
      }),
    );

    expect(result2.current.currentWorkflowIndex).toBe(0);
  });
});
