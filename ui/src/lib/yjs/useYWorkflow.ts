import { XYPosition } from "@xyflow/react";
import { useCallback } from "react";
import * as Y from "yjs";
import { Array as YArray } from "yjs";

import { config } from "@flow/config";
import {
  DEFAULT_ENTRY_GRAPH_ID,
  DEFAULT_ROUTING_PORT,
} from "@flow/global-constants";
import type { Action, Edge, Node } from "@flow/types";
import { generateUUID } from "@flow/utils";

import { fetcher } from "../fetch/transformers/useFetch";

import { yNodeConstructor, yWorkflowConstructor } from "./conversions";
import type { YNode, YNodesArray, YWorkflow } from "./types";

export default ({
  yWorkflows,
  rawWorkflows,
  currentWorkflowId,
  undoTrackerActionWrapper,
}: {
  yWorkflows: YArray<YWorkflow>;
  rawWorkflows: Record<string, string | Node[] | Edge[]>[];
  currentWorkflowId: string;
  undoTrackerActionWrapper: (callback: () => void) => void;
}) => {
  const { api } = config();
  const currentYWorkflow = yWorkflows.get(
    rawWorkflows.findIndex((w) => w.id === currentWorkflowId) || 0,
  );

  const fetchRouterConfigs = useCallback(async () => {
    const [inputRouter, outputRouter] = await Promise.all([
      fetcher<Action>(`${api}/actions/InputRouter`),
      fetcher<Action>(`${api}/actions/OutputRouter`),
    ]);
    return { inputRouter, outputRouter };
  }, [api]);

  const createYWorkflow = useCallback(
    (
      workflowId: string,
      workflowName: string,
      position: XYPosition,

      routers: { inputRouter: Action; outputRouter: Action },
      initialNodes?: Node[],
      initialEdges?: Edge[],
    ) => {
      const { inputRouter, outputRouter } = routers;

      const inputNodeId = generateUUID();
      // newInputNode is not a YNode because it will be converted in the yWorkflowConstructor
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
      // newOutputNode is not a YNode because it will be converted in the yWorkflowConstructor
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

      const workflowNodes = [
        newInputNode,
        ...(initialNodes ?? []),
        newOutputNode,
      ];
      const newYWorkflow = yWorkflowConstructor(
        workflowId,
        workflowName,
        workflowNodes,
        initialEdges,
      );

      const newSubworkflowNode: YNode = yNodeConstructor({
        id: workflowId,
        type: "subworkflow",
        position,
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
      });

      return { newYWorkflow, newSubworkflowNode };
    },
    [],
  );

  const handleYWorkflowAdd = useCallback(
    async (position: XYPosition = { x: 600, y: 200 }) => {
      try {
        const routers = await fetchRouterConfigs();
        undoTrackerActionWrapper(() => {
          const workflowId = generateUUID();
          const workflowName = `Sub Workflow-${yWorkflows.length}`;

          const { newYWorkflow, newSubworkflowNode } = createYWorkflow(
            workflowId,
            workflowName,
            position,
            routers,
          );

          const parentWorkflow = yWorkflows.get(
            rawWorkflows.findIndex((w) => w.id === currentWorkflowId) || 0,
          );
          const parentWorkflowNodes = parentWorkflow?.get("nodes") as
            | YNodesArray
            | undefined;
          parentWorkflowNodes?.insert(parentWorkflowNodes.length, [
            newSubworkflowNode,
          ]);

          yWorkflows.insert(yWorkflows.length, [newYWorkflow]);
        });
      } catch (error) {
        console.error("Failed to add workflow:", error);
        throw error;
      }
    },
    [
      yWorkflows,
      currentWorkflowId,
      rawWorkflows,
      createYWorkflow,
      fetchRouterConfigs,
      undoTrackerActionWrapper,
    ],
  );

  const handleYWorkflowAddFromSelection = useCallback(
    async (nodes: Node[], edges: Edge[]) => {
      try {
        const routers = await fetchRouterConfigs();

        undoTrackerActionWrapper(() => {
          const nodesByParentId = new Map<string, Node[]>();
          nodes.forEach((node) => {
            if (node.parentId) {
              if (!nodesByParentId.has(node.parentId)) {
                nodesByParentId.set(node.parentId, []);
              }
              nodesByParentId.get(node.parentId)?.push(node);
            }
          });

          const selectedNodes = nodes.filter((n) => n.selected);
          if (selectedNodes.length === 0) return;

          const getBatchNodes = (batchId: string): Node[] =>
            nodesByParentId.get(batchId) ?? [];

          const allIncludedNodeIds = new Set<string>();
          selectedNodes.forEach((node) => {
            allIncludedNodeIds.add(node.id);
            if (node.type === "batch") {
              getBatchNodes(node.id).forEach((batchNode) =>
                allIncludedNodeIds.add(batchNode.id),
              );
            }
          });

          const allIncludedNodes = nodes.filter((n) =>
            allIncludedNodeIds.has(n.id),
          );
          const position = {
            x: Math.min(...selectedNodes.map((n) => n.position.x)),
            y: Math.min(...selectedNodes.map((n) => n.position.y)),
          };

          const adjustedNodes = allIncludedNodes.map((node) => ({
            ...node,
            position: node.parentId
              ? node.position
              : {
                  x: node.position.x - position.x + 400,
                  y: node.position.y - position.y + 200,
                },
            selected: false,
          }));

          const internalEdges = edges.filter(
            (e) =>
              allIncludedNodeIds.has(e.source) &&
              allIncludedNodeIds.has(e.target),
          );

          const workflowId = generateUUID();
          const workflowName = `Sub Workflow-${yWorkflows.length}`;

          const { newYWorkflow, newSubworkflowNode } = createYWorkflow(
            workflowId,
            workflowName,
            position,
            routers,
            adjustedNodes,
            internalEdges,
          );

          const parentWorkflow = yWorkflows.get(
            rawWorkflows.findIndex((w) => w.id === currentWorkflowId) || 0,
          );
          const parentWorkflowNodes = parentWorkflow?.get("nodes") as
            | YNodesArray
            | undefined;
          const remainingNodes = nodes
            .filter((n) => !allIncludedNodeIds.has(n.id))
            .map((n) => yNodeConstructor(n));

          parentWorkflowNodes?.delete(0, parentWorkflowNodes.length);
          parentWorkflowNodes?.insert(0, [
            ...remainingNodes,
            newSubworkflowNode,
          ]);

          yWorkflows.insert(yWorkflows.length, [newYWorkflow]);
        });
      } catch (error) {
        console.error("Failed to add workflow from selection:", error);
        throw error;
      }
    },
    [
      yWorkflows,
      currentWorkflowId,
      rawWorkflows,
      createYWorkflow,
      fetchRouterConfigs,
      undoTrackerActionWrapper,
    ],
  );

  const handleYWorkflowUpdate = useCallback(
    (workflowId: string, nodes?: Node[], edges?: Edge[]) => {
      const workflowName = "Sub Workflow-" + yWorkflows.length.toString();
      const newYWorkflow = yWorkflowConstructor(
        workflowId,
        workflowName,
        nodes,
        edges,
      );
      yWorkflows.insert(yWorkflows.length, [newYWorkflow]);
    },
    [yWorkflows],
  );

  const handleYWorkflowsRemove = useCallback(
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
      }),
    [rawWorkflows, yWorkflows, undoTrackerActionWrapper],
  );

  const handleYWorkflowRename = useCallback(
    (id: string, name: string) =>
      undoTrackerActionWrapper(() => {
        if (!name.trim()) {
          throw new Error("Workflow name cannot be empty");
        }

        // Update subworkflow node in main workflow if this is a subworkflow
        const mainWorkflow = yWorkflows.get(0);
        const mainWorkflowNodes = mainWorkflow?.get("nodes") as YNodesArray;

        for (const node of mainWorkflowNodes) {
          // Get the id from the YNode
          const nodeId = (node.get("id") as Y.Text).toString();

          if (nodeId === id) {
            // Get existing data as YMap
            const nodeData = node.get("data") as Y.Map<unknown>;

            if (nodeData.get("customName")?.toString() === name) return;
            nodeData.set("customName", name);
          }
        }
      }),
    [undoTrackerActionWrapper, yWorkflows],
  );

  return {
    currentYWorkflow,
    handleYWorkflowAdd,
    handleYWorkflowUpdate,
    handleYWorkflowsRemove,
    handleYWorkflowRename,
    handleYWorkflowAddFromSelection,
  };
};
