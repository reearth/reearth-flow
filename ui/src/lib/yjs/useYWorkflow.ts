import { XYPosition } from "@xyflow/react";
import { Dispatch, SetStateAction, useCallback } from "react";
import { Array as YArray } from "yjs";

import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import type { Edge, Node } from "@flow/types";
import { randomID } from "@flow/utils";

import { YNodesArray, YWorkflow, yWorkflowBuilder } from "./utils";

export default ({
  yWorkflows,
  rawWorkflows,
  currentWorkflowIndex,
  undoTrackerActionWrapper,
  setWorkflows,
  setOpenWorkflowIds,
}: {
  yWorkflows: YArray<YWorkflow>;
  rawWorkflows: Record<string, string | Node[] | Edge[]>[];
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
}) => {
  const currentYWorkflow = yWorkflows.get(currentWorkflowIndex);

  const handleWorkflowAdd = useCallback(
    (position?: XYPosition) =>
      undoTrackerActionWrapper(() => {
        const workflowId = randomID();
        const workflowName = "Sub Workflow-" + yWorkflows.length.toString();

        const newEntranceNode: Node = {
          id: randomID(),
          type: "entrance",
          position: { x: 200, y: 200 },
          data: {
            name: `New Entrance node`,
            outputs: ["target"],
            status: "idle",
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
      }),
    [
      yWorkflows,
      currentWorkflowIndex,
      undoTrackerActionWrapper,
      setOpenWorkflowIds,
      setWorkflows,
    ],
  );

  const handleWorkflowUpdate = useCallback(
    (workflowId: string, nodes?: Node[], edges?: Edge[]) => {
      const workflowName = "Sub Workflow-" + yWorkflows.length.toString();
      const newYWorkflow = yWorkflowBuilder(
        workflowId,
        workflowName,
        nodes,
        edges,
      );
      yWorkflows.push([newYWorkflow]);
      setWorkflows((w) => [...w, { id: workflowId, name: workflowName }]);
    },
    [setWorkflows, yWorkflows],
  );

  const handleWorkflowsRemove = useCallback(
    (nodeIds: string[]) =>
      undoTrackerActionWrapper(() => {
        const workflowIds: string[] = [];

        const removeNodes = (nodeIds: string[]) => {
          nodeIds.forEach((nid) => {
            if (nid === DEFAULT_ENTRY_GRAPH_ID) return;

            const index = rawWorkflows.findIndex((w) => w.id === nid);
            if (index === -1) return;

            // Loop over workflow at current index and remove any subworkflow nodes
            (rawWorkflows[index].nodes as Node[]).forEach((node) => {
              if (node.type === "subworkflow") {
                removeNodes([node.id]);
              }
            });

            workflowIds.push(nid);
            yWorkflows.delete(index);
          });
        };

        removeNodes(nodeIds);

        setWorkflows((w) => w.filter((w) => !workflowIds.includes(w.id)));
        setOpenWorkflowIds((ids) =>
          ids.filter((id) => !workflowIds.includes(id)),
        );
      }),
    [
      rawWorkflows,
      yWorkflows,
      undoTrackerActionWrapper,
      setWorkflows,
      setOpenWorkflowIds,
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
    handleWorkflowUpdate,
    handleWorkflowsRemove,
    handleWorkflowRename,
  };
};
