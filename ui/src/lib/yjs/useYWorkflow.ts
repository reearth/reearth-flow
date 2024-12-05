import { XYPosition } from "@xyflow/react";
import { Dispatch, SetStateAction, useCallback } from "react";
import { Array as YArray } from "yjs";

import type { Node } from "@flow/types";
import { randomID } from "@flow/utils";

import { YNodesArray, YWorkflow, yWorkflowBuilder } from "./utils";

export default ({
  yWorkflows,
  workflows,
  currentWorkflowIndex,
  undoTrackerActionWrapper,
  setWorkflows,
  setOpenWorkflowIds,
  handleWorkflowIdChange,
}: {
  yWorkflows: YArray<YWorkflow>;
  workflows: {
    id: string;
    name: string;
  }[];
  currentWorkflowIndex: number;
  undoTrackerActionWrapper: (callback: () => void) => void;
  setWorkflows: Dispatch<
    SetStateAction<
      {
        id: string;
        name: string;
      }[]
    >
  >;
  setOpenWorkflowIds: Dispatch<SetStateAction<string[]>>;
  handleWorkflowIdChange: (id?: string) => void;
}) => {
  const currentYWorkflow = yWorkflows.get(currentWorkflowIndex);

  const handleWorkflowAdd = useCallback(
    (position?: XYPosition) =>
      undoTrackerActionWrapper(() => {
        const workflowId = yWorkflows.length.toString() + "-workflow";
        const workflowName = "Sub Workflow-" + yWorkflows.length.toString();

        const newEntranceNode: Node = {
          id: randomID(),
          type: "entrance",
          position: { x: 200, y: 200 },
          data: {
            name: `New Entrance node`,
            outputs: ["target"],
            status: "idle",
            // locked: false,
            // onLock: onNodeLocking,
          },
        };

        const newExitNode: Node = {
          id: randomID(),
          type: "exit",
          position: { x: 1000, y: 200 },
          data: {
            name: `New Exit node`,
            inputs: ["source"],
            status: "idle",
            // locked: false,
            // onLock: onNodeLocking,
          },
        };

        const newYWorkflow = yWorkflowBuilder(workflowId, workflowName, [
          newEntranceNode,
          newExitNode,
        ]);

        // Update main workflow
        const newSubworkflowNode: Node = {
          id: workflowId,
          type: "subworkflow",
          position: position ?? { x: 600, y: 200 },
          data: {
            name: workflowName,
            status: "idle",
            inputs: ["source"],
            outputs: ["target"],
          },
        };

        const parentWorkflow = yWorkflows.get(currentWorkflowIndex ?? 0);

        const parentWorkflowNodes = parentWorkflow?.get("nodes") as
          | YNodesArray
          | undefined;
        parentWorkflowNodes?.push([newSubworkflowNode]);

        yWorkflows.push([newYWorkflow]);
        setWorkflows((w) => [...w, { id: workflowId, name: workflowName }]);
        setOpenWorkflowIds((ids) => [...ids, workflowId]);

        handleWorkflowIdChange(workflowId);
      }),
    [
      yWorkflows,
      currentWorkflowIndex,
      undoTrackerActionWrapper,
      setOpenWorkflowIds,
      setWorkflows,
      handleWorkflowIdChange,
    ],
  );

  const handleWorkflowsRemove = useCallback(
    (workflowIds: string[]) =>
      undoTrackerActionWrapper(() => {
        workflowIds.forEach((wid) => {
          if (wid === "main") return;
          const index = workflows.findIndex((w) => w.id === wid);
          if (index === -1) return;
          if (index === currentWorkflowIndex) {
            handleWorkflowIdChange("main");
          }
          yWorkflows.delete(index);
        });

        setWorkflows((w) => w.filter((w) => !workflowIds.includes(w.id)));
        setOpenWorkflowIds((ids) =>
          ids.filter((id) => !workflowIds.includes(id)),
        );
      }),
    [
      workflows,
      yWorkflows,
      currentWorkflowIndex,
      undoTrackerActionWrapper,
      setWorkflows,
      setOpenWorkflowIds,
      handleWorkflowIdChange,
    ],
  );

  const handleWorkflowRename = useCallback(
    (id: string, name: string) =>
      undoTrackerActionWrapper(() => {
        if (!name.trim()) {
          throw new Error("Workflow name cannot be empty");
        }

        // Update local state
        setWorkflows((w) => w.map((w) => (w.id === id ? { ...w, name } : w)));

        // Update subworkflow node in main workflow if this is a subworkflow
        const mainWorkflow = yWorkflows.get(0);
        const mainWorkflowNodes = mainWorkflow?.get("nodes") as YNodesArray;

        for (const node of mainWorkflowNodes) {
          if (node.id === id) {
            node.data = {
              ...node.data,
              name,
            };
          }
        }
      }),
    [undoTrackerActionWrapper, yWorkflows, setWorkflows],
  );

  return {
    currentYWorkflow,
    handleWorkflowAdd,
    handleWorkflowsRemove,
    handleWorkflowRename,
  };
};
