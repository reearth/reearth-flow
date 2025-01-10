import { Dispatch, SetStateAction, useCallback } from "react";
import { Array as YArray } from "yjs";

import { config } from "@flow/config";
import { DEFAULT_ROUTING_PORT } from "@flow/global-constants";
import { fetcher } from "@flow/lib/fetch/transformers/useFetch";
import { YNodesArray, YWorkflow, yWorkflowBuilder } from "@flow/lib/yjs/utils";
import { Node, Edge, Action } from "@flow/types";
import { generateUUID } from "@flow/utils";

export const useSubworkflowCreation = ({
  nodes,
  edges,
  yWorkflows,
  rawWorkflows,
  currentWorkflowId,
  undoTrackerActionWrapper,
  setOpenWorkflowIds,
  setWorkflows,
}: {
  nodes: Node[];
  edges: Edge[];
  yWorkflows: YArray<YWorkflow>;
  rawWorkflows: Record<string, string | Node[] | Edge[]>[];
  currentWorkflowId: string;
  undoTrackerActionWrapper: (callback: () => void) => void;
  setOpenWorkflowIds: Dispatch<SetStateAction<string[]>>;
  setWorkflows: Dispatch<SetStateAction<{ id: string; name: string }[]>>; // Add this
}) => {
  const { api } = config();
  const handleCreateSubworkflow = useCallback(async () => {
    undoTrackerActionWrapper(async () => {
      const selectedNodes = nodes.filter((n) => n.selected);
      const selectedNodeIds = selectedNodes.map((n) => n.id);

      if (selectedNodes.length === 0) return;

      const workflowId = generateUUID();
      const position = {
        x: Math.min(...selectedNodes.map((n) => n.position.x)),
        y: Math.min(...selectedNodes.map((n) => n.position.y)),
      };

      const internalEdges = edges.filter(
        (e) =>
          selectedNodeIds.includes(e.source) &&
          selectedNodeIds.includes(e.target),
      );

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
      const outputRouter = await fetcher<Action>(`${api}/actions/OutputRouter`);

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

      const adjustedNodes = selectedNodes.map((node) => ({
        ...node,
        position: {
          x: node.position.x - position.x + 400,
          y: node.position.y - position.y + 200,
        },
        selected: false,
      }));

      const allNodes = [newInputNode, ...adjustedNodes, newOutputNode];
      const workflowName = `Sub Workflow-${yWorkflows.length}`;
      const newYWorkflow = yWorkflowBuilder(
        workflowId,
        workflowName,
        allNodes,
        internalEdges,
      );

      const newSubworkflowNode: Node = {
        id: workflowId,
        type: "subworkflow",
        position,
        data: {
          officialName: `Sub Workflow-${yWorkflows.length}`,
          status: "idle",
          pseudoInputs: [
            { nodeId: inputNodeId, portName: DEFAULT_ROUTING_PORT },
          ],
          pseudoOutputs: [
            { nodeId: outputNodeId, portName: DEFAULT_ROUTING_PORT },
          ],
        },
        selected: true,
      };
      yWorkflows.push([newYWorkflow]);

      const parentWorkflow = yWorkflows.get(
        rawWorkflows.findIndex((w) => w.id === currentWorkflowId) || 0,
      );

      const parentWorkflowNodes = parentWorkflow?.get("nodes") as
        | YNodesArray
        | undefined;
      parentWorkflowNodes?.push([newSubworkflowNode]);

      const remainingNodes = nodes.filter(
        (n) => !selectedNodeIds.includes(n.id),
      );

      parentWorkflowNodes?.delete(0, parentWorkflowNodes.length);
      parentWorkflowNodes?.push([...remainingNodes, newSubworkflowNode]);
      setWorkflows((w) => [...w, { id: workflowId, name: workflowName }]);
      setOpenWorkflowIds((ids) => [...ids, workflowId]);
    });
  }, [
    nodes,
    edges,
    yWorkflows,
    rawWorkflows,
    currentWorkflowId,
    api,
    undoTrackerActionWrapper,
    setOpenWorkflowIds,
    setWorkflows,
  ]);

  return {
    handleCreateSubworkflow,
  };
};
