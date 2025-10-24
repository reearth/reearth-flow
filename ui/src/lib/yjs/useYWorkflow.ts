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
  yEdgeConstructor,
  yNodeConstructor,
  yWorkflowConstructor,
} from "./conversions";
import type { YNode, YNodesMap, YEdgesMap, YWorkflow } from "./types";

export default ({
  yWorkflows,
  currentWorkflowId,
  undoTrackerActionWrapper,
}: {
  yWorkflows: YMap<YWorkflow>;
  currentWorkflowId: string;
  undoTrackerActionWrapper: (
    callback: () => void,
    originPrepend?: string,
  ) => void;
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
        ...(initialNodes ?? [newInputNode, newOutputNode]),
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
            | YNodesMap
            | undefined;
          parentWorkflowNodes?.set(workflowId, newSubworkflowNode);

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

          const boundaryEdges = edges.filter((edge) => {
            const sourceSelected = allIncludedNodeIds.has(edge.source);
            const targetSelected = allIncludedNodeIds.has(edge.target);
            return sourceSelected !== targetSelected;
          });

          const workflowId = generateUUID();
          const workflowName = t("Subworkflow");

          const additionalRouterNodes: Node[] = [];
          const additionalInternalEdges: Edge[] = [];
          const externalEdges: Edge[] = [];
          const pseudoInputs: { nodeId: string; portName: string }[] = [];
          const pseudoOutputs: { nodeId: string; portName: string }[] = [];

          // Track routers by external connection (external node + handle) to avoid duplicates
          const inputRoutersByExternalConnection = new Map<
            string,
            { routerId: string; portName: string; externalEdge: Edge }
          >();
          const outputRoutersByExternalConnection = new Map<
            string,
            { routerId: string; portName: string; externalEdge: Edge }
          >();

          let inputCounter = 1;
          let outputCounter = 1;

          boundaryEdges.forEach((edge) => {
            const sourceSelected = allIncludedNodeIds.has(edge.source);
            const targetSelected = allIncludedNodeIds.has(edge.target);

            if (!sourceSelected && targetSelected) {
              // Group by external source node + its source handle
              const externalConnectionKey = `${edge.source}:${edge.sourceHandle ?? "default"}`;

              let inputRouterInfo = inputRoutersByExternalConnection.get(
                externalConnectionKey,
              );

              if (!inputRouterInfo) {
                const inputRouterId = generateUUID();
                const portName = `${DEFAULT_ROUTING_PORT}-${inputCounter}`;
                inputCounter++;

                const inputRouter: Node = {
                  id: inputRouterId,
                  type: routers.inputRouter.type as NodeType,
                  position: {
                    x: 200 + (inputCounter - 2) * 50,
                    y: 150,
                  },
                  data: {
                    officialName: routers.inputRouter.name,
                    outputs: routers.inputRouter.outputPorts,
                    params: { routingPort: portName },
                  },
                };
                additionalRouterNodes.push(inputRouter);

                const externalEdge: Edge = {
                  ...edge,
                  id: edge.id,
                  target: workflowId,
                  targetHandle: portName,
                };

                inputRouterInfo = {
                  routerId: inputRouterId,
                  portName,
                  externalEdge,
                };
                inputRoutersByExternalConnection.set(
                  externalConnectionKey,
                  inputRouterInfo,
                );

                pseudoInputs.push({
                  nodeId: inputRouterId,
                  portName: portName,
                });
              }

              const internalEdge: Edge = {
                id: generateUUID(),
                source: inputRouterInfo.routerId,
                target: edge.target,
                targetHandle: edge.targetHandle,
              };
              additionalInternalEdges.push(internalEdge);
            } else if (sourceSelected && !targetSelected) {
              // Group by external target node + its target handle
              const externalConnectionKey = `${edge.target}:${edge.targetHandle ?? "default"}`;

              let outputRouterInfo = outputRoutersByExternalConnection.get(
                externalConnectionKey,
              );

              if (!outputRouterInfo) {
                const outputRouterId = generateUUID();
                const portName = `${DEFAULT_ROUTING_PORT}-${outputCounter}`;
                outputCounter++;

                const outputRouter: Node = {
                  id: outputRouterId,
                  type: routers.outputRouter.type as NodeType,
                  position: {
                    x: 800 + (outputCounter - 2) * 50,
                    y: 150,
                  },
                  data: {
                    officialName: routers.outputRouter.name,
                    inputs: routers.outputRouter.inputPorts,
                    params: { routingPort: portName },
                  },
                };
                additionalRouterNodes.push(outputRouter);

                const externalEdge: Edge = {
                  ...edge,
                  id: edge.id,
                  source: workflowId,
                  sourceHandle: portName,
                };

                outputRouterInfo = {
                  routerId: outputRouterId,
                  portName,
                  externalEdge,
                };
                outputRoutersByExternalConnection.set(
                  externalConnectionKey,
                  outputRouterInfo,
                );

                pseudoOutputs.push({
                  nodeId: outputRouterId,
                  portName: portName,
                });
              }

              const internalEdge: Edge = {
                id: generateUUID(),
                source: edge.source,
                target: outputRouterInfo.routerId,
                sourceHandle: edge.sourceHandle,
              };
              additionalInternalEdges.push(internalEdge);
            }
          });

          // Collect unique external edges (one per router)
          inputRoutersByExternalConnection.forEach((info) => {
            externalEdges.push(info.externalEdge);
          });
          outputRoutersByExternalConnection.forEach((info) => {
            externalEdges.push(info.externalEdge);
          });

          const allSubworkflowNodes = [
            ...adjustedNodes,
            ...additionalRouterNodes,
          ];

          const allSubworkflowEdges = [
            ...internalEdges,
            ...additionalInternalEdges,
          ];

          const { newYWorkflow, newSubworkflowNode } = createYWorkflow(
            workflowId,
            workflowName,
            position,
            routers,
            allSubworkflowNodes,
            allSubworkflowEdges,
          );

          const parentWorkflow = currentYWorkflow;
          const parentWorkflowNodesMap = parentWorkflow?.get("nodes") as
            | YNodesMap
            | undefined;
          const parentWorkflowEdgesMap = parentWorkflow?.get("edges") as
            | YEdgesMap
            | undefined;

          allIncludedNodeIds.forEach((nodeId) => {
            parentWorkflowNodesMap?.delete(nodeId);
          });

          // Delete internal edges that are moving to the subworkflow
          internalEdges.forEach((edge) => {
            parentWorkflowEdgesMap?.delete(edge.id);
          });

          // Delete boundary edges that are being replaced with router connections
          boundaryEdges.forEach((edge) => {
            parentWorkflowEdgesMap?.delete(edge.id);
          });

          externalEdges.forEach((edge) => {
            const yEdge = yEdgeConstructor(edge);
            parentWorkflowEdgesMap?.set(edge.id, yEdge);
          });

          parentWorkflowNodesMap?.set(workflowId, newSubworkflowNode);

          if (pseudoInputs.length > 0 || pseudoOutputs.length > 0) {
            const subworkflowNodeInParent =
              parentWorkflowNodesMap?.get(workflowId);

            if (subworkflowNodeInParent) {
              const nodeData = subworkflowNodeInParent.get(
                "data",
              ) as Y.Map<any>;
              const existingPseudoInputs = nodeData?.get(
                "pseudoInputs",
              ) as Y.Array<any>;
              const existingPseudoOutputs = nodeData?.get(
                "pseudoOutputs",
              ) as Y.Array<any>;
              // Clear existing pseudo inputs/outputs as it is easier to create them from scratch than attempt to connect to the existing ones
              if (existingPseudoInputs) {
                existingPseudoInputs.delete(0, existingPseudoInputs.length);
              }
              if (existingPseudoOutputs) {
                existingPseudoOutputs.delete(0, existingPseudoOutputs.length);
              }

              pseudoInputs.forEach((pseudoInput) => {
                const yPseudoInput = new Y.Map();
                yPseudoInput.set("nodeId", new Y.Text(pseudoInput.nodeId));
                yPseudoInput.set("portName", new Y.Text(pseudoInput.portName));
                existingPseudoInputs?.push([yPseudoInput]);
              });

              pseudoOutputs.forEach((pseudoOutput) => {
                const yPseudoOutput = new Y.Map();
                yPseudoOutput.set("nodeId", new Y.Text(pseudoOutput.nodeId));
                yPseudoOutput.set(
                  "portName",
                  new Y.Text(pseudoOutput.portName),
                );
                existingPseudoOutputs?.push([yPseudoOutput]);
              });
            }
          }

          yWorkflows.set(workflowId, newYWorkflow);
        });
      } catch (error) {
        console.error("Failed to add workflow from selection:", error);
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
        const yWorkflow = yWorkflows.get(id);
        if (!yWorkflow) return;
        const yName = new Y.Text();
        yName.insert(0, name.trim());
        yWorkflow.set("name", yName);
      }),
    [undoTrackerActionWrapper, yWorkflows],
  );

  return {
    currentYWorkflow,
    handleYWorkflowAdd,
    handleYWorkflowUpdate,
    handleYWorkflowRemove,
    handleYWorkflowRename,
    handleYWorkflowAddFromSelection,
  };
};
