import { renderHook } from "@testing-library/react";
import { useY } from "react-yjs";
import * as Y from "yjs";

import useWorkflowTabs from "./useWorkflowTabs";
import { YWorkflow, yWorkflowBuilder } from "./workflowBuilder";

describe("useWorkflowTabs", () => {
  it("should initialize with the first workflow as active", () => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
    const yWorkflow = yWorkflowBuilder("1", "Workflow-1");
    yWorkflows.push([yWorkflow]);

    const workflowId = "1";
    const { result: result1 } = renderHook(() => useY(yWorkflows));

    const { result: result2 } = renderHook(() =>
      useWorkflowTabs({
        workflowId,
        rawWorkflows: result1.current,
        handleWorkflowIdChange: vi.fn(),
      }),
    );
    console.log("result2", result2.current);

    expect(result2.current.currentWorkflowIndex).toBe(0);
    expect(result2.current.workflows[0].id).toBe("1");
  });
});
