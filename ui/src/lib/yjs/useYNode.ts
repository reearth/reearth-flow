import type { Dispatch, SetStateAction } from "react";
import { useCallback, useRef } from "react";
import * as Y from "yjs";

import { applySchemaPatch } from "@flow/features/Editor/usePreviewSchema/schemaReport";
import { DEFAULT_ROUTING_PORT } from "@flow/global-constants";
import type {
  Node,
  NodeChange,
  NodeMetadata,
  NodeSchemaMeta,
  Workflow,
} from "@flow/types";

import { yNodeConstructor } from "./conversions";
import type { YNode, YNodesMap, YNodeValue, YWorkflow } from "./types";
import { updateParentYWorkflow } from "./useParentYWorkflow";
import { removeParentYWorkflowNodePseudoPort } from "./useParentYWorkflow/removeParentYWorkflowNodePseudoPort";
import { computeWorkflowPath } from "./utils/computeWorkflowPath";

// Y.Map.set records a new item even when the value is identical, and every
// item is a document update. React Flow reports dimensions for every node each
// time it measures them, and the same position again at the end of a drag, so
// only the values that actually differ are written.
const setYNodeValues = (
  yNode: YNode,
  key: string,
  values: Record<string, number | string>,
) => {
  const existing = yNode.get(key);
  if (existing instanceof Y.Map) {
    for (const [k, v] of Object.entries(values)) {
      if (existing.get(k) !== v) existing.set(k, v);
    }
    return;
  }
  const yMap = new Y.Map<unknown>();
  for (const [k, v] of Object.entries(values)) yMap.set(k, v);
  yNode.set(key, yMap);
};

export default ({
  currentYWorkflow,
  yWorkflows,
  rawWorkflows,
  setSelectedNodeIds,
  undoTrackerActionWrapper,
  handleYWorkflowRemove,
}: {
  currentYWorkflow?: YWorkflow;
  yWorkflows: Y.Map<YWorkflow>;
  rawWorkflows: Workflow[];
  setSelectedNodeIds: Dispatch<SetStateAction<string[]>>;
  undoTrackerActionWrapper: (
    callback: () => void,
    originPrepend?: string,
  ) => void;
  handleYWorkflowRemove?: (workflowId: string) => void;
}) => {
  const handleYNodesAdd = useCallback(
    (newNodes: Node[]) => {
      undoTrackerActionWrapper(() => {
        const yNodes = currentYWorkflow?.get("nodes") as YNodesMap | undefined;
        if (!yNodes) return;

        const selectedNewNodeIds = newNodes
          .filter((n) => n.selected)
          .map((n) => n.id);
        if (selectedNewNodeIds.length > 0) {
          setSelectedNodeIds(selectedNewNodeIds);
        }

        newNodes.forEach((newNode) => {
          const isRouterInput = newNode.data.officialName === "Input Router";
          const isRouterOutput = newNode.data.officialName === "Output Router";

          if (isRouterInput || isRouterOutput) {
            const currentWorkflowId = currentYWorkflow
              ?.get("id")
              ?.toJSON() as string;
            const parentWorkflow = rawWorkflows.find((w) => {
              const nodes = w.nodes as Node[];
              return nodes.some(
                (n) => n.data.subworkflowId === currentWorkflowId,
              );
            });

            const basePortName =
              newNode.data.params?.routingPort || DEFAULT_ROUTING_PORT;
            let uniquePortName = basePortName;

            if (parentWorkflow) {
              const parentYWorkflow = yWorkflows.get(parentWorkflow.id);
              if (parentYWorkflow) {
                const parentYNodes = parentYWorkflow.get("nodes") as YNodesMap;
                const parentNodes = Object.values(
                  parentYNodes.toJSON(),
                ) as Node[];
                const subworkflowNode = parentNodes.find(
                  (n) => n.data.subworkflowId === currentWorkflowId,
                );

                if (subworkflowNode) {
                  const existingPorts = isRouterInput
                    ? subworkflowNode.data.pseudoInputs || []
                    : subworkflowNode.data.pseudoOutputs || [];

                  const existingPortNames = new Set(
                    existingPorts.map((p) => p.portName),
                  );

                  let counter = 1;
                  while (existingPortNames.has(uniquePortName)) {
                    uniquePortName = `${basePortName}-${counter}`;
                    counter++;
                  }
                }
              }
            }

            newNode.data.params = {
              ...newNode.data.params,
              routingPort: uniquePortName,
            };
          }
          const currentWorkflowId = currentYWorkflow?.get("id")?.toString();
          if (currentWorkflowId) {
            newNode.data.workflowPath = computeWorkflowPath(
              rawWorkflows,
              currentWorkflowId,
            );
          }

          yNodes.set(newNode.id, yNodeConstructor(newNode));

          // Update parent pseudoports if this is a router with routingPort
          if (
            (isRouterInput || isRouterOutput) &&
            newNode.data.params?.routingPort
          ) {
            const currentWorkflowId = currentYWorkflow
              ?.get("id")
              ?.toJSON() as string;
            const parentWorkflow = rawWorkflows.find((w) => {
              const nodes = w.nodes as Node[];
              return nodes.some(
                (n) => n.data.subworkflowId === currentWorkflowId,
              );
            });

            if (parentWorkflow) {
              const parentYWorkflow = yWorkflows.get(parentWorkflow.id);
              if (parentYWorkflow) {
                updateParentYWorkflow(
                  currentWorkflowId,
                  parentYWorkflow,
                  newNode,
                  newNode.data.params,
                );
              }
            }
          }
        });
      });
    },
    [
      currentYWorkflow,
      setSelectedNodeIds,
      undoTrackerActionWrapper,
      rawWorkflows,
      yWorkflows,
    ],
  );

  // Passed to editor context so needs to be a ref
  const handleYNodesChangeRef =
    useRef<(changes: NodeChange[]) => void>(undefined);
  // This is based off of react-flow node changes, which includes removal
  // but not addtion. This is why we have a separate function for adding nodes.
  handleYNodesChangeRef.current = (changes: NodeChange[]) => {
    const yNodes = currentYWorkflow?.get("nodes") as YNodesMap | undefined;
    if (!yNodes) return;

    undoTrackerActionWrapper(() => {
      changes.forEach((change) => {
        switch (change.type) {
          case "position": {
            const existingYNode = yNodes.get(change.id);

            if (existingYNode && change.position) {
              setYNodeValues(existingYNode, "position", {
                x: change.position.x,
                y: change.position.y,
              });
            }
            break;
          }
          case "replace": {
            const existingYNode = yNodes.get(change.id);

            if (existingYNode && change.item) {
              const newYNode = yNodeConstructor(change.item);
              yNodes.set(change.id, newYNode);
            }
            break;
          }
          case "dimensions": {
            const existingYNode = yNodes.get(change.id);

            if (existingYNode && change.dimensions) {
              const { width, height } = change.dimensions;
              setYNodeValues(existingYNode, "measured", { width, height });

              if (change.setAttributes) {
                setYNodeValues(existingYNode, "style", {
                  width: width + "px",
                  height: height + "px",
                });
              }
            }
            break;
          }
          case "remove": {
            const existingYNode = yNodes.get(change.id);

            if (existingYNode) {
              const nodeToDelete = existingYNode.toJSON() as Node;
              if (
                nodeToDelete.type === "subworkflow" &&
                nodeToDelete.data.subworkflowId
              ) {
                handleYWorkflowRemove?.(nodeToDelete.data.subworkflowId);
              } else if (nodeToDelete.data.params?.routingPort) {
                const parentWorkflowId = rawWorkflows.find((w) => {
                  const nodes = w.nodes as Node[];
                  return nodes.some(
                    (n) =>
                      n.id ===
                      (currentYWorkflow?.get("id")?.toJSON() as string),
                  );
                })?.id;
                if (!parentWorkflowId) return;
                const parentYWorkflow = yWorkflows.get(parentWorkflowId);
                if (parentYWorkflow) {
                  removeParentYWorkflowNodePseudoPort(
                    currentYWorkflow?.get("id")?.toJSON() as string,
                    parentYWorkflow,
                    nodeToDelete,
                  );
                }
              }

              setSelectedNodeIds((snids) => {
                return snids.filter((snid) => snid !== change.id);
              });

              yNodes.delete(change.id);
            }
            break;
          }
          case "select": {
            setSelectedNodeIds((snids) => {
              if (change.selected) {
                return [...snids, change.id];
              } else {
                return snids.filter((snid) => snid !== change.id);
              }
            });
            break;
          }
        }
      });
    });
  };
  const handleYNodesChange = useCallback(
    (changes: NodeChange[]) => handleYNodesChangeRef.current?.(changes),
    [],
  );

  const handleYNodesDataUpdate = useCallback(
    (
      nodesToChange: {
        nodeId: string;
        type?: string;
        updatedParams?: any;
        updatedCustomizations?: any;
        paramsSchema?: any;
        isDisabled?: boolean;
      }[],
    ) =>
      undoTrackerActionWrapper(() => {
        const yNodes = currentYWorkflow?.get("nodes") as YNodesMap | undefined;
        if (!yNodes) return;

        const nodes = Object.values(yNodes.toJSON()) as Node[];

        nodesToChange.forEach(
          ({
            nodeId,
            type,
            updatedParams,
            updatedCustomizations,
            paramsSchema,
            isDisabled,
          }) => {
            const prevNode = nodes.find((n) => n.id === nodeId);

            if (!prevNode) return;
            // if params.routingPort exists, it's parent is a subworkflow and
            // we need to update pseudoInputs and pseudoOutputs on the parent node.
            if (updatedParams?.routingPort) {
              const currentWorkflowId = currentYWorkflow
                ?.get("id")
                ?.toJSON() as string;

              const parentWorkflow = rawWorkflows.find((w) => {
                const nodes = w.nodes as Node[];
                return nodes.some(
                  (n) => n.data.subworkflowId === currentWorkflowId,
                );
              });
              if (!parentWorkflow) return;
              const parentYWorkflow = yWorkflows.get(parentWorkflow.id);
              if (!parentYWorkflow) return;

              updateParentYWorkflow(
                currentWorkflowId,
                parentYWorkflow,
                prevNode,
                updatedParams,
              );
            }

            const yData = yNodes.get(nodeId)?.get("data") as Y.Map<YNodeValue>;
            if (updatedParams) yData?.set("params", updatedParams);
            if (updatedCustomizations)
              yData?.set("customizations", updatedCustomizations);
            if (paramsSchema !== undefined)
              yData?.set("paramsSchema", paramsSchema as any);

            if (type === "batch" && isDisabled !== undefined) {
              const nodesByParentId = new Map<string, Node[]>();
              nodes.forEach((node) => {
                if (node.parentId) {
                  if (!nodesByParentId.has(node.parentId)) {
                    nodesByParentId.set(node.parentId, []);
                  }
                  nodesByParentId.get(node.parentId)?.push(node);
                }
              });

              const getChildNodes = (batchId: string): Node[] =>
                nodesByParentId.get(batchId) ?? [];

              const childNodes = getChildNodes(nodeId);
              childNodes.forEach((childNode) => {
                const childYData = yNodes
                  .get(childNode.id)
                  ?.get("data") as Y.Map<YNodeValue>;
                if (childYData) {
                  childYData.set("isDisabled", isDisabled);
                }
              });
            }

            if (isDisabled !== undefined) {
              yData?.set("isDisabled", isDisabled);
            }
          },
        );
      }),
    [currentYWorkflow, rawWorkflows, yWorkflows, undoTrackerActionWrapper],
  );

  const handleYNodeSchemaUpdate = useCallback(
    (nodeId: string, patch: Partial<NodeSchemaMeta> | undefined) =>
      undoTrackerActionWrapper(() => {
        // A probe can finish after the user has switched to another workflow,
        // so look for the node in every workflow, current one first.
        const yNode =
          (currentYWorkflow?.get("nodes") as YNodesMap | undefined)?.get(
            nodeId,
          ) ??
          Array.from(yWorkflows.values())
            .map((w) => (w.get("nodes") as YNodesMap | undefined)?.get(nodeId))
            .find(Boolean);
        const yData = yNode?.get("data") as Y.Map<any> | undefined;
        if (!yData) return;
        const prevMetadata =
          (yData.get("nodeMetadata") as NodeMetadata | undefined) ?? {};
        yData.set("nodeMetadata", {
          ...prevMetadata,
          schema: applySchemaPatch(prevMetadata.schema, patch),
        });
      }, "schema-update"),
    [currentYWorkflow, yWorkflows, undoTrackerActionWrapper],
  );

  return {
    handleYNodesAdd,
    handleYNodesChange,
    handleYNodesDataUpdate,
    handleYNodeSchemaUpdate,
  };
};
