import { renderHook } from "@testing-library/react";
import { useY } from "react-yjs";
import * as Y from "yjs";

import useWorkflowTabs from "./useWorkflowTabs";
import { YWorkflow, yWorkflowBuilder } from "./utils";

describe("useWorkflowTabs", () => {
  it("should initialize with the first workflow as active", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
    const yWorkflow = yWorkflowBuilder("1", "Workflow-1");
    yWorkflows.push([yWorkflow]);

    const currentWorkflowId = "1";
    const { result: result1 } = renderHook(() => useY(yWorkflows));

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
