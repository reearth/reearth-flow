import { XYPosition } from "@xyflow/react";
import { Dispatch, SetStateAction, useCallback } from "react";
import { Array as YArray } from "yjs";

import { config } from "@flow/config";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import type { Action, Edge, Node } from "@flow/types";
import { randomID } from "@flow/utils";

import { fetcher } from "../fetch/transformers/useFetch";

import { YNodesArray, YWorkflow, yWorkflowBuilder } from "./utils";

export default ({
  yWorkflows,
  rawWorkflows,
  currentWorkflowId,
  undoTrackerActionWrapper,
  setWorkflows,
  setOpenWorkflowIds,
}: {
  yWorkflows: YArray<YWorkflow>;
  rawWorkflows: Record<string, string | Node[] | Edge[]>[];
  currentWorkflowId: string;
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
  const { api } = config();
  const currentYWorkflow = yWorkflows.get(
    rawWorkflows.findIndex((w) => w.id === currentWorkflowId) || 0,
  );

  const handleWorkflowAdd = useCallback(
    (position?: XYPosition) =>
      undoTrackerActionWrapper(async () => {
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

        const routerNode = await fetcher<Action>(`${api}/actions/Router`);

        const newExitNode: Node = {
          id: randomID(),
          type: routerNode.type,
          position: { x: 1000, y: 200 },
          data: {
            name: routerNode.name,
            inputs: routerNode.inputPorts,
            status: "idle",
          },
        };

        const newYWorkflow = yWorkflowBuilder(workflowId, workflowName, [
          newEntranceNode,
          newExitNode,
        ]);

        const newSubworkflowNode: Node = {
          id: workflowId,
          type: "subworkflow",
          position: position ?? { x: 600, y: 200 },
          data: {
            name: workflowName,
            status: "idle",
            inputs: ["source"],
            outputs: routerNode.outputPorts,
          },
        };

        const parentWorkflow = yWorkflows.get(
          rawWorkflows.findIndex((w) => w.id === currentWorkflowId) || 0,
        );

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
      currentWorkflowId,
      rawWorkflows,
      api,
      undoTrackerActionWrapper,
      setWorkflows,
      setOpenWorkflowIds,
    ],
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
    handleWorkflowsRemove,
    handleWorkflowRename,
  };
};
