import { XYPosition } from "@xyflow/react";
import { Dispatch, SetStateAction, useCallback } from "react";
import { Array as YArray } from "yjs";

import { config } from "@flow/config";
import {
  DEFAULT_ENTRY_GRAPH_ID,
  DEFAULT_ROUTING_PORT,
} from "@flow/global-constants";
import type { Action, Edge, Node } from "@flow/types";
import { generateUUID } from "@flow/utils";

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
        const workflowId = generateUUID();
        const workflowName = "Sub Workflow-" + yWorkflows.length.toString();

        const inputRouter = await fetcher<Action>(`${api}/actions/InputRouter`);

        const inputNodeId = generateUUID();
        const newInputNode: Node = {
          id: inputNodeId,
          type: inputRouter.type,
          position: { x: 200, y: 200 },
          data: {
            officialName: inputRouter.name,
            outputs: inputRouter.outputPorts,
            status: "idle",
            params: {
              routingPort: DEFAULT_ROUTING_PORT,
            },
          },
        };

        const outputRouter = await fetcher<Action>(
          `${api}/actions/OutputRouter`,
        );

        const outputNodeId = generateUUID();
        const newOutputNode: Node = {
          id: outputNodeId,
          type: outputRouter.type,
          position: { x: 1000, y: 200 },
          data: {
            officialName: outputRouter.name,
            inputs: outputRouter.inputPorts,
            status: "idle",
            params: {
              routingPort: DEFAULT_ROUTING_PORT,
            },
          },
        };

        const newYWorkflow = yWorkflowBuilder(workflowId, workflowName, [
          newInputNode,
          newOutputNode,
        ]);

        const newSubworkflowNode: Node = {
          id: workflowId,
          type: "subworkflow",
          position: position ?? { x: 600, y: 200 },
          data: {
            officialName: workflowName,
            status: "idle",
            pseudoInputs: [
              { nodeId: inputNodeId, portName: DEFAULT_ROUTING_PORT },
            ],
            pseudoOutputs: [
              { nodeId: outputNodeId, portName: DEFAULT_ROUTING_PORT },
            ],
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

        const collectWorkflowIds = (nid: string) => {
          if (nid === DEFAULT_ENTRY_GRAPH_ID) return;

          const workflow = rawWorkflows.find((w) => w.id === nid);
          if (!workflow) return;

          workflowIds.push(nid);

          const nodes = workflow.nodes;
          // Loop over workflow and remove any subworkflow nodes
          if (nodes && Array.isArray(nodes)) {
            (nodes as Node[]).forEach((node) => {
              if (node.type === "subworkflow") {
                collectWorkflowIds(node.id);
              }
            });
          }
        };

        nodeIds.forEach((nid) => collectWorkflowIds(nid));
        // Indexes in descending order to avoid index shifting problems
        const indexesToRemove = workflowIds
          .map((id) => rawWorkflows.findIndex((w) => w.id === id))
          .filter((index) => index !== -1)
          .sort((a, b) => b - a);

        indexesToRemove.forEach((index) => {
          yWorkflows.delete(index);
          if (Array.isArray(rawWorkflows)) {
            rawWorkflows.splice(index, 1);
          }
        });

        setWorkflows((currentWorkflows) => {
          const remainingWorkflows = currentWorkflows.filter(
            (w) => !workflowIds.includes(w.id),
          );
          return remainingWorkflows;
        });

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
              customName: name,
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
