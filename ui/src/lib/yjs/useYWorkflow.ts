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
    (position?: XYPosition, nodes?: Node[], edges?: Edge[]) =>
      undoTrackerActionWrapper(async () => {
        const workflowId = generateUUID();
        const workflowName = `Sub Workflow-${yWorkflows.length}`;

        const nodesByParentId = new Map<string, Node[]>();
        nodes?.forEach((node) => {
          if (node.parentId) {
            if (!nodesByParentId.has(node.parentId)) {
              nodesByParentId.set(node.parentId, []);
            }
            nodesByParentId.get(node.parentId)?.push(node);
          }
        });

        const selectedNodes = nodes?.filter((n) => n.selected);
        const hasSelectedNodes = selectedNodes && selectedNodes.length > 0;

        const getBatchNodes = (batchId: string): Node[] =>
          nodesByParentId.get(batchId) ?? [];

        // Get all nodes that should be included (selected nodes + their nested nodes)
        const allIncludedNodeIds = new Set<string>();
        selectedNodes?.forEach((node) => {
          allIncludedNodeIds.add(node.id);
          if (node.type === "batch") {
            getBatchNodes(node.id).forEach((batchNode) =>
              allIncludedNodeIds.add(batchNode.id),
            );
          }
        });

        const allIncludedNodes =
          nodes?.filter((n) => allIncludedNodeIds.has(n.id)) ?? [];

        // Calculate position for new subworkflow node
        const calculatedPosition = hasSelectedNodes
          ? {
              x: Math.min(...selectedNodes.map((n) => n.position.x)),
              y: Math.min(...selectedNodes.map((n) => n.position.y)),
            }
          : (position ?? { x: 600, y: 200 });

        const [inputRouter, outputRouter] = await Promise.all([
          fetcher<Action>(`${api}/actions/InputRouter`),
          fetcher<Action>(`${api}/actions/OutputRouter`),
        ]);

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

        let workflowNodes = [newInputNode, newOutputNode];
        let workflowEdges: Edge[] = [];

        if (hasSelectedNodes && edges) {
          const internalEdges = edges.filter(
            (e) =>
              allIncludedNodeIds.has(e.source) &&
              allIncludedNodeIds.has(e.target),
          );

          // Adjust positions of included nodes, respecting parentId (batch nodes)
          const adjustedNodes = allIncludedNodes.map((node) => {
            if (node.parentId) {
              return {
                ...node,
                selected: false,
              };
            }
            return {
              ...node,
              position: {
                x: node.position.x - calculatedPosition.x + 400,
                y: node.position.y - calculatedPosition.y + 200,
              },
              selected: false,
            };
          });

          workflowNodes = [newInputNode, ...adjustedNodes, newOutputNode];
          workflowEdges = internalEdges;
        }

        const newYWorkflow = yWorkflowBuilder(
          workflowId,
          workflowName,
          workflowNodes,
          workflowEdges,
        );

        const newSubworkflowNode: Node = {
          id: workflowId,
          type: "subworkflow",
          position: calculatedPosition,
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
          selected: true,
        };

        const parentWorkflow = yWorkflows.get(
          rawWorkflows.findIndex((w) => w.id === currentWorkflowId) || 0,
        );
        const parentWorkflowNodes = parentWorkflow?.get("nodes") as
          | YNodesArray
          | undefined;

        if (hasSelectedNodes) {
          const remainingNodes = nodes?.filter(
            (n) => !allIncludedNodeIds.has(n.id),
          );
          parentWorkflowNodes?.delete(0, parentWorkflowNodes.length);
          parentWorkflowNodes?.push([
            ...(remainingNodes ?? []),
            newSubworkflowNode,
          ]);
        } else {
          parentWorkflowNodes?.push([newSubworkflowNode]);
        }

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
        const localWorkflows = [...rawWorkflows];

        const removeNodes = (nodeIds: string[]) => {
          nodeIds.forEach((nid) => {
            if (nid === DEFAULT_ENTRY_GRAPH_ID) return;

            const index = localWorkflows.findIndex((w) => w.id === nid);
            if (index === -1) return;

            // Loop over workflow at current index and remove any subworkflow nodes
            (localWorkflows[index].nodes as Node[]).forEach((node) => {
              if (node.type === "subworkflow") {
                removeNodes([node.id]);
              }
            });

            workflowIds.push(nid);
            yWorkflows.delete(index);
            localWorkflows.splice(index, 1);
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
