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
      needsDefaultRouters?: boolean,
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

      const workflowNodes = needsDefaultRouters
        ? [newInputNode, ...(initialNodes ?? []), newOutputNode]
        : [...(initialNodes ?? [newInputNode, newOutputNode])];
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
          const selectedNodes = nodes.filter((n) => n.selected);
          const containsReadersOrWriters = selectedNodes?.some(
            (n) => n.type === "reader" || n.type === "writer",
          );
          if (containsReadersOrWriters) {
            return;
          }
          const nodesByParentId = new Map<string, Node[]>();
          nodes.forEach((node) => {
            if (node.parentId) {
              if (!nodesByParentId.has(node.parentId)) {
                nodesByParentId.set(node.parentId, []);
              }
              nodesByParentId.get(node.parentId)?.push(node);
            }
          });

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
          const externalEdgesMap = new Map<string, Edge>();
          const edgesToDelete = new Set<string>();
          const pseudoInputs: { nodeId: string; portName: string }[] = [];
          const pseudoOutputs: { nodeId: string; portName: string }[] = [];

          // First pass: identify which node names appear multiple times in selected nodes
          const nodeNameCounts = new Map<string, number>();
          const nodeInstanceNumbers = new Map<string, number>();

          allIncludedNodes.forEach((node) => {
            const officialName = node.data.officialName || node.id;
            nodeNameCounts.set(
              officialName,
              (nodeNameCounts.get(officialName) || 0) + 1,
            );
          });

          // Assign instance numbers to nodes whose names appear multiple times
          const nodeNameCounters = new Map<string, number>();
          allIncludedNodes.forEach((node) => {
            const officialName = node.data.officialName || node.id;
            const count = nodeNameCounts.get(officialName) || 0;

            if (count > 1) {
              const instanceNum = (nodeNameCounters.get(officialName) || 0) + 1;
              nodeNameCounters.set(officialName, instanceNum);
              nodeInstanceNumbers.set(node.id, instanceNum);
            }
          });

          // Track routers by INTERNAL connection (one router per internal node+handle)
          const inputRoutersByInternalTarget = new Map<
            string,
            { routerId: string; portName: string }
          >();
          const outputRoutersByInternalSource = new Map<
            string,
            { routerId: string; portName: string }
          >();

          // Helper function to create a meaningful handle name
          const getHandleName = (
            node: Node | undefined,
            handle: string | undefined | null,
          ): string => {
            if (!handle) {
              return node?.data.officialName || node?.id || "default";
            }
            return handle;
          };

          // First pass: count how many external sources connect to each internal target
          // and how many external targets each internal source connects to
          const inputConnectionCounts = new Map<string, number>();
          const outputConnectionCounts = new Map<string, number>();

          boundaryEdges.forEach((edge) => {
            const sourceSelected = allIncludedNodeIds.has(edge.source);
            const targetSelected = allIncludedNodeIds.has(edge.target);
            const nodeSource = nodes.find((n) => n.id === edge.source);
            const nodeTarget = nodes.find((n) => n.id === edge.target);

            if (!sourceSelected && targetSelected) {
              // Key by INTERNAL target node + handle (not external source)
              const targetHandleName = getHandleName(
                nodeTarget,
                edge.targetHandle,
              );
              const internalTargetKey = `${edge.target}:${targetHandleName}`;
              inputConnectionCounts.set(
                internalTargetKey,
                (inputConnectionCounts.get(internalTargetKey) || 0) + 1,
              );
            } else if (sourceSelected && !targetSelected) {
              // Key by INTERNAL source node + handle (not external target)
              const sourceHandleName = getHandleName(
                nodeSource,
                edge.sourceHandle,
              );
              const internalSourceKey = `${edge.source}:${sourceHandleName}`;
              outputConnectionCounts.set(
                internalSourceKey,
                (outputConnectionCounts.get(internalSourceKey) || 0) + 1,
              );
            }
          });

          boundaryEdges.forEach((edge) => {
            const sourceSelected = allIncludedNodeIds.has(edge.source);
            const targetSelected = allIncludedNodeIds.has(edge.target);
            const nodeSource = nodes.find((n) => n.id === edge.source);
            const nodeTarget = nodes.find((n) => n.id === edge.target);

            // --- INPUT ROUTERS (external -> internal) ---
            if (!sourceSelected && targetSelected) {
              // Key by INTERNAL target - one router per internal node+handle
              const targetHandleName = getHandleName(
                nodeTarget,
                edge.targetHandle,
              );
              const internalTargetKey = `${edge.target}:${targetHandleName}`;
              let inputRouterInfo =
                inputRoutersByInternalTarget.get(internalTargetKey);

              if (!inputRouterInfo) {
                const inputRouterId = generateUUID();

                let portName: string;

                // Use target node name + handle
                const targetNodeName =
                  nodeTarget?.data.officialName ?? nodeTarget?.id ?? "input";
                const instanceNum = nodeInstanceNumbers.get(edge.target);

                portName = targetNodeName;
                if (instanceNum !== undefined) {
                  portName += `-${instanceNum}`;
                }
                portName += `-${edge.targetHandle}`;

                const inputRouter: Node = {
                  id: inputRouterId,
                  type: routers.inputRouter.type as NodeType,
                  position: {
                    x: 200 + inputRoutersByInternalTarget.size * 50,
                    y: 150,
                  },
                  data: {
                    officialName: routers.inputRouter.name,
                    outputs: routers.inputRouter.outputPorts,
                    params: { routingPort: portName },
                  },
                };

                additionalRouterNodes.push(inputRouter);
                inputRouterInfo = { routerId: inputRouterId, portName };
                inputRoutersByInternalTarget.set(
                  internalTargetKey,
                  inputRouterInfo,
                );
                pseudoInputs.push({ nodeId: inputRouterId, portName });
              }

              // Mark this boundary edge for deletion
              edgesToDelete.add(edge.id);

              // Create external edge (one per external source, connecting to the subworkflow)
              const externalEdgeKey = `${edge.source}:${getHandleName(nodeSource, edge.sourceHandle)}-to-${edge.target}`;
              if (!externalEdgesMap.has(externalEdgeKey)) {
                const externalEdge: Edge = {
                  ...edge,
                  id: edge.id,
                  target: workflowId,
                  targetHandle: inputRouterInfo.portName,
                };
                externalEdgesMap.set(externalEdgeKey, externalEdge);
              }

              // Internal connection (router -> internal node)
              const internalEdge: Edge = {
                id: generateUUID(),
                source: inputRouterInfo.routerId,
                target: edge.target,
                targetHandle: edge.targetHandle,
              };
              additionalInternalEdges.push(internalEdge);
            }

            // --- OUTPUT ROUTERS (internal -> external) ---
            else if (sourceSelected && !targetSelected) {
              // Key by INTERNAL source - one router per internal node+handle
              const sourceHandleName = getHandleName(
                nodeSource,
                edge.sourceHandle,
              );
              const internalSourceKey = `${edge.source}:${sourceHandleName}`;
              let outputRouterInfo =
                outputRoutersByInternalSource.get(internalSourceKey);

              if (!outputRouterInfo) {
                const outputRouterId = generateUUID();

                let portName: string;

                // Use source node name + handle
                const sourceNodeName =
                  nodeSource?.data.officialName ?? nodeSource?.id ?? "output";
                const instanceNum = nodeInstanceNumbers.get(edge.source);

                portName = sourceNodeName;
                if (instanceNum !== undefined) {
                  portName += `-${instanceNum}`;
                }
                portName += `-${edge.sourceHandle}`;

                const outputRouter: Node = {
                  id: outputRouterId,
                  type: routers.outputRouter.type as NodeType,
                  position: {
                    x: 800 + outputRoutersByInternalSource.size * 50,
                    y: 150,
                  },
                  data: {
                    officialName: routers.outputRouter.name,
                    inputs: routers.outputRouter.inputPorts,
                    params: { routingPort: portName },
                  },
                };

                additionalRouterNodes.push(outputRouter);
                outputRouterInfo = { routerId: outputRouterId, portName };
                outputRoutersByInternalSource.set(
                  internalSourceKey,
                  outputRouterInfo,
                );
                pseudoOutputs.push({ nodeId: outputRouterId, portName });
              }

              // Mark this boundary edge for deletion
              edgesToDelete.add(edge.id);

              // Create external edge (one per external target, connecting from the subworkflow)
              const externalEdgeKey = `${edge.source}-to-${edge.target}:${getHandleName(nodeTarget, edge.targetHandle)}`;
              if (!externalEdgesMap.has(externalEdgeKey)) {
                const externalEdge: Edge = {
                  ...edge,
                  id: edge.id,
                  source: workflowId,
                  sourceHandle: outputRouterInfo.portName,
                };
                externalEdgesMap.set(externalEdgeKey, externalEdge);
              }

              // Internal connection (internal node -> router)
              const internalEdge: Edge = {
                id: generateUUID(),
                source: edge.source,
                target: outputRouterInfo.routerId,
                sourceHandle: edge.sourceHandle,
              };
              additionalInternalEdges.push(internalEdge);
            }
          });

          const needsDefaultRouters = boundaryEdges.length === 0;

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
            needsDefaultRouters,
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

          // Delete all boundary edges first (they're being replaced by router connections)
          edgesToDelete.forEach((edgeId) => {
            parentWorkflowEdgesMap?.delete(edgeId);
          });

          // Add the new external edges (one per unique external connection)
          externalEdgesMap.forEach((edge) => {
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
