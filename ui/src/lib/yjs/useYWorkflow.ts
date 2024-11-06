import { Dispatch, SetStateAction, useCallback } from "react";
import { Array as YArray, Text as YText } from "yjs";

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
  handleWorkflowOpen,
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
  handleWorkflowOpen: (workflowId: string) => void;
}) => {
  const currentYWorkflow = yWorkflows.get(currentWorkflowIndex);

  const handleWorkflowAdd = useCallback(
    () =>
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
          position: { x: 600, y: 200 },
          data: {
            name: workflowName,
            status: "idle",
            inputs: ["source"],
            outputs: ["target"],
            onDoubleClick: handleWorkflowOpen,
          },
        };
        const mainWorkflow = yWorkflows.get(0);

        const mainWorkflowNodes = mainWorkflow?.get("nodes") as
          | YNodesArray
          | undefined;
        mainWorkflowNodes?.push([newSubworkflowNode]);

        yWorkflows.push([newYWorkflow]);
        setWorkflows((w) => [...w, { id: workflowId, name: workflowName }]);
        setOpenWorkflowIds((ids) => [...ids, workflowId]);
      }),
    [
      yWorkflows,
      undoTrackerActionWrapper,
      setOpenWorkflowIds,
      setWorkflows,
      handleWorkflowOpen,
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

        const workflowIndex = workflows.findIndex((w) => w.id === id);

        // Update Yjs shared data
        const workflow = yWorkflows.get(workflowIndex);
        if (!workflow) {
          throw new Error("Workflow not found");
        }
        workflow.set("name", new YText(name));

        // Update subworkflow node in main workflow if this is a subworkflow
        if (workflowIndex > 0) {
          const mainWorkflow = yWorkflows.get(0);
          const mainWorkflowNodes = mainWorkflow?.get("nodes") as YNodesArray;

          mainWorkflowNodes?.forEach((node, index) => {
            if (node.id === id) {
              const updatedNode = {
                ...node,
                data: { ...node.data, name },
              };
              mainWorkflowNodes.delete(index);
              mainWorkflowNodes.insert(index, [updatedNode]);
            }
          });
        }
      }),
    [undoTrackerActionWrapper, yWorkflows, workflows, setWorkflows],
  );

  return {
    currentYWorkflow,
    handleWorkflowAdd,
    handleWorkflowsRemove,
    handleWorkflowRename,
  };
};
