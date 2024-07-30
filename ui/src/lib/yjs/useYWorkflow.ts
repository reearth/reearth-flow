import { Dispatch, SetStateAction, useCallback } from "react";
import { Array as YArray } from "yjs";

import type { Node } from "@flow/types";
import { randomID } from "@flow/utils";

import { YNodesArray, YWorkflow, yWorkflowBuilder } from "./workflowBuilder";

export default ({
  yWorkflows,
  workflows,
  currentWorkflowIndex,
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

  const handleWorkflowAdd = useCallback(() => {
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

    const newYWorkflow = yWorkflowBuilder(workflowId, workflowName, [newEntranceNode, newExitNode]);

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

    const mainWorkflowNodes = mainWorkflow?.get("nodes") as YNodesArray | undefined;
    mainWorkflowNodes?.push([newSubworkflowNode]);

    yWorkflows.push([newYWorkflow]);
    setWorkflows(w => [...w, { id: workflowId, name: workflowName }]);
    setOpenWorkflowIds(ids => [...ids, workflowId]);
  }, [yWorkflows, setOpenWorkflowIds, setWorkflows, handleWorkflowOpen]);

  const handleWorkflowRemove = useCallback(
    (workflowId: string) => {
      const index = workflows.findIndex(w => w.id === workflowId);
      setWorkflows(w => w.filter(w => w.id !== workflowId));

      if (index === currentWorkflowIndex) {
        handleWorkflowIdChange("main");
      }
      yWorkflows.delete(index);

      // Remove subworkflow node from main workflow
      const mainWorkflow = yWorkflows.get(0);
      const mainWorkflowNodes = mainWorkflow?.get("nodes") as YNodesArray | undefined;
      const subworkflowIndex = mainWorkflowNodes
        ?.toJSON()
        .findIndex((n: Node) => n.id === workflowId);
      if (subworkflowIndex !== undefined && subworkflowIndex !== -1) {
        mainWorkflowNodes?.delete(subworkflowIndex);
      }
    },
    [workflows, yWorkflows, currentWorkflowIndex, setWorkflows, handleWorkflowIdChange],
  );
  return {
    currentYWorkflow,
    handleWorkflowAdd,
    handleWorkflowRemove,
  };
};
