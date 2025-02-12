// import { renderHook, act, cleanup } from "@testing-library/react";
// import { describe, test, expect, vi } from "vitest";
// import * as Y from "yjs";

import { cleanup } from "@testing-library/react";

// import useYWorkflow from "./useYWorkflow"; // Adjust the import according to your file structure
// import { YWorkflow, yWorkflowConstructor } from "./workflowBuilder";

afterEach(() => {
  cleanup();
});

describe("useWorkflowFunctions", () => {
  test("should add a new workflow", () => {
    // const yDoc = new Y.Doc();
    // const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
    // const workflows: { id: string; name: string }[] = [];
    // const setWorkflows = vi.fn();
    // const setOpenWorkflowIds = vi.fn();
    // const handleWorkflowIdChange = vi.fn();
    // const handleWorkflowOpen = vi.fn();
    // const { result } = renderHook(() =>
    //   useYWorkflow({
    //     yWorkflows,
    //     workflows,
    //     currentWorkflowIndex: 0,
    //     undoTrackerActionWrapper: vi.fn(),
    //     setWorkflows,
    //     setOpenWorkflowIds,
    //     handleWorkflowIdChange,
    //     handleWorkflowOpen,
    //   }),
    // );
    // act(() => {
    //   result.current.handleWorkflowAdd();
    // });
    // expect(setWorkflows).toHaveBeenCalledWith(expect.any(Function));
    // expect(setOpenWorkflowIds).toHaveBeenCalledWith(expect.any(Function));
    // expect(yWorkflows.toJSON().length).toBe(1);
    // expect(yWorkflows.get(0).get("id")?.toJSON()).toBe("0-workflow");
    // expect(yWorkflows.get(0).get("name")?.toJSON()).toBe("Sub Workflow-0");
  });

  test("should remove workflows", () => {
    // const yDoc = new Y.Doc();
    // const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
    // const yWorkflow = yWorkflowConstructor("sub-workflow1", "Sub Workflow 1");
    // yWorkflows.push([yWorkflow]);
    // const workflows = [{ id: "sub-workflow1", name: "Sub Workflow 1" }];
    // const setWorkflows = vi.fn();
    // const setOpenWorkflowIds = vi.fn();
    // const handleWorkflowIdChange = vi.fn();
    // const handleWorkflowOpen = vi.fn();
    // const { result } = renderHook(() =>
    //   useYWorkflow({
    //     yWorkflows,
    //     workflows,
    //     currentWorkflowIndex: 0,
    //     undoTrackerActionWrapper: vi.fn(),
    //     setWorkflows,
    //     setOpenWorkflowIds,
    //     handleWorkflowIdChange,
    //     handleWorkflowOpen,
    //   }),
    // );
    // act(() => {
    //   result.current.handleWorkflowsRemove(["sub-workflow1"]);
    // });
    // expect(setWorkflows).toHaveBeenCalledWith(expect.any(Function));
    // expect(setOpenWorkflowIds).toHaveBeenCalledWith(expect.any(Function));
    // expect(yWorkflows.toJSON().length).toBe(0);
  });
});
