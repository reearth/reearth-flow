import { XYPosition } from "@xyflow/react";
import { useCallback } from "react";
import * as Y from "yjs";
import { Map as YMap } from "yjs";

import { config } from "@flow/config";
import {
  DEFAULT_ENTRY_GRAPH_ID,
  DEFAULT_ROUTING_PORT,
} from "@flow/global-constants";
import { fetcher } from "@flow/lib/fetch/transformers/useFetch";
import { useT } from "@flow/lib/i18n";
import type { Action, Edge, Node, NodeType } from "@flow/types";
import { generateUUID, isDefined } from "@flow/utils";

import {
  rebuildWorkflow,
  yNodeConstructor,
  yWorkflowConstructor,
} from "./conversions";
import type { YNode, YNodesArray, YWorkflow } from "./types";

export default ({
  yWorkflows,
  currentWorkflowId,
  undoTrackerActionWrapper,
}: {
  yWorkflows: YMap<YWorkflow>;
  currentWorkflowId: string;
  undoTrackerActionWrapper: (callback: () => void) => void;
}) => {
  const t = useT();
  const { api } = config();
  const currentYWorkflow = yWorkflows.get(currentWorkflowId);

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
        type: inputRouter.type as NodeType,
        position: { x: 200, y: 200 },
        data: {
          officialName: inputRouter.name,
          outputs: inputRouter.outputPorts,
          params: {
            routingPort: DEFAULT_ROUTING_PORT,
          },
        },
      };

      const outputNodeId = generateUUID();
      // newOutputNode is not a YNode because it will be converted in the yWorkflowConstructor
      const newOutputNode: Node = {
        id: outputNodeId,
        type: outputRouter.type as NodeType,
        position: { x: 1000, y: 200 },
        data: {
          officialName: outputRouter.name,
          inputs: outputRouter.inputPorts,
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
          pseudoInputs: [
            { nodeId: inputNodeId, portName: DEFAULT_ROUTING_PORT },
          ],
          pseudoOutputs: [
            { nodeId: outputNodeId, portName: DEFAULT_ROUTING_PORT },
          ],
          subworkflowId: workflowId,
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
          const workflowName = t("Subworkflow");

          const { newYWorkflow, newSubworkflowNode } = createYWorkflow(
            workflowId,
            workflowName,
            position,
            routers,
          );

          const parentWorkflow = currentYWorkflow;
          const parentWorkflowNodes = parentWorkflow?.get("nodes") as
            | YNodesArray
            | undefined;
          parentWorkflowNodes?.insert(parentWorkflowNodes.length, [
            newSubworkflowNode,
          ]);

          yWorkflows.set(workflowId, newYWorkflow);
        });
      } catch (error) {
        console.error("Failed to add workflow:", error);
        throw error;
      }
    },
    [
      yWorkflows,
      currentYWorkflow,
      t,
      createYWorkflow,
      fetchRouterConfigs,
      undoTrackerActionWrapper,
    ],
  );

  // const handleYWorkflowAddFromSelection = useCallback(
  //   async (nodes: Node[], edges: Edge[]) => {
  //     try {
  //       const routers = await fetchRouterConfigs();

  //       undoTrackerActionWrapper(() => {
  //         const nodesByParentId = new Map<string, Node[]>();
  //         nodes.forEach((node) => {
  //           if (node.parentId) {
  //             if (!nodesByParentId.has(node.parentId)) {
  //               nodesByParentId.set(node.parentId, []);
  //             }
  //             nodesByParentId.get(node.parentId)?.push(node);
  //           }
  //         });

  //         const selectedNodes = nodes.filter((n) => n.selected);
  //         if (selectedNodes.length === 0) return;

  //         const getBatchNodes = (batchId: string): Node[] =>
  //           nodesByParentId.get(batchId) ?? [];

  //         const allIncludedNodeIds = new Set<string>();
  //         selectedNodes.forEach((node) => {
  //           allIncludedNodeIds.add(node.id);
  //           if (node.type === "batch") {
  //             getBatchNodes(node.id).forEach((batchNode) =>
  //               allIncludedNodeIds.add(batchNode.id),
  //             );
  //           }
  //         });

  //         const allIncludedNodes = nodes.filter((n) =>
  //           allIncludedNodeIds.has(n.id),
  //         );
  //         const position = {
  //           x: Math.min(...selectedNodes.map((n) => n.position.x)),
  //           y: Math.min(...selectedNodes.map((n) => n.position.y)),
  //         };

  //         const adjustedNodes = allIncludedNodes.map((node) => ({
  //           ...node,
  //           position: node.parentId
  //             ? node.position
  //             : {
  //                 x: node.position.x - position.x + 400,
  //                 y: node.position.y - position.y + 200,
  //               },
  //           selected: false,
  //         }));

  //         const internalEdges = edges.filter(
  //           (e) =>
  //             allIncludedNodeIds.has(e.source) &&
  //             allIncludedNodeIds.has(e.target),
  //         );

  //         const workflowId = generateUUID();
  //         const workflowName = t("Subworkflow");

  //         const { newYWorkflow, newSubworkflowNode } = createYWorkflow(
  //           workflowId,
  //           workflowName,
  //           position,
  //           routers,
  //           adjustedNodes,
  //           internalEdges,
  //         );

  //         const parentWorkflow = currentYWorkflow;
  //         const parentWorkflowNodes = parentWorkflow?.get("nodes") as
  //           | YNodesArray
  //           | undefined;

  //         const parentWorkflowEdges = parentWorkflow?.get("edges") as
  //           | YEdgesMap
  //           | undefined;

  //         const remainingNodes = nodes
  //           .filter((n) => !allIncludedNodeIds.has(n.id))
  //           .map((n) => yNodeConstructor(n));

  //         const remainingEdges = edges
  //           .filter(
  //             (e) =>
  //               !allIncludedNodeIds.has(e.source) ||
  //               !allIncludedNodeIds.has(e.target),
  //           )
  //           .map((e) => yEdgeConstructor(e));

  //         parentWorkflowEdges?.delete(0, parentWorkflowEdges.length);
  //         parentWorkflowNodes?.delete(0, parentWorkflowNodes.length);
  //         parentWorkflowNodes?.insert(0, [
  //           ...remainingNodes,
  //           newSubworkflowNode,
  //         ]);
  //         parentWorkflowEdges?.insert(0, remainingEdges);

  //         yWorkflows.set(workflowId, newYWorkflow);
  //       });
  //     } catch (error) {
  //       console.error("Failed to add workflow from selection:", error);
  //       throw error;
  //     }
  //   },
  //   [
  //     yWorkflows,
  //     currentYWorkflow,
  //     t,
  //     createYWorkflow,
  //     fetchRouterConfigs,
  //     undoTrackerActionWrapper,
  //   ],
  // );

  const handleYWorkflowUpdate = useCallback(
    (workflowId: string, nodes?: Node[], edges?: Edge[]) =>
      undoTrackerActionWrapper(() => {
        const workflowName = t("Subworkflow");
        const newYWorkflow = yWorkflowConstructor(
          workflowId,
          workflowName,
          nodes,
          edges,
        );
        yWorkflows.set(workflowId, newYWorkflow);
      }),
    [yWorkflows, t, undoTrackerActionWrapper],
  );

  const handleYWorkflowRemove = useCallback(
    (workflowId: string) =>
      undoTrackerActionWrapper(() => {
        const workflowsToRemove = new Set<string>();

        const markWorkflowForRemoval = (id: string) => {
          if (id === DEFAULT_ENTRY_GRAPH_ID) return;
          if (workflowsToRemove.has(id)) return; // Avoid circular references

          workflowsToRemove.add(id);

          const yWorkflow = yWorkflows.get(id);
          if (!yWorkflow) return;
          const workflow = rebuildWorkflow(yWorkflow);

          (workflow.nodes as Node[]).forEach((node) => {
            if (node.type === "subworkflow" && node.data.subworkflowId) {
              markWorkflowForRemoval(node.data.subworkflowId);
            }
          });
        };

        markWorkflowForRemoval(workflowId);

        const idsToRemove = Array.from(workflowsToRemove)
          .map((id) => id)
          .filter(isDefined);

        idsToRemove.forEach((id) => {
          yWorkflows.delete(id);
        });
      }),
    [yWorkflows, undoTrackerActionWrapper],
  );

  const handleYWorkflowRename = useCallback(
    (id: string, name: string) =>
      undoTrackerActionWrapper(() => {
        if (!name.trim()) {
          throw new Error("Workflow name cannot be empty");
        }

        // Update subworkflow node in main workflow if this is a subworkflow
        const mainWorkflow = yWorkflows.get(DEFAULT_ENTRY_GRAPH_ID);
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
    handleYWorkflowRemove,
    handleYWorkflowRename,
    handleYWorkflowAddFromSelection: undefined,
  };
};
