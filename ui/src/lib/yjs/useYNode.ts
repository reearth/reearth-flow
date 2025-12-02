import { Dispatch, SetStateAction, useCallback, useRef } from "react";
import * as Y from "yjs";

import { DEFAULT_ROUTING_PORT } from "@flow/global-constants";
import type { Node, NodeChange, Workflow } from "@flow/types";

import { yNodeConstructor } from "./conversions";
import type { YNodesMap, YNodeValue, YWorkflow } from "./types";
import { updateParentYWorkflow } from "./useParentYWorkflow";
import { removeParentYWorkflowNodePseudoPort } from "./useParentYWorkflow/removeParentYWorkflowNodePseudoPort";

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

        newNodes.forEach((newNode) => {
          if (newNode.selected) {
            setSelectedNodeIds((snids) => {
              return [...snids, newNode.id];
            });
          }

          // For routers without routingPort, generate unique port name
          const isRouterInput = newNode.data.officialName === "InputRouter";
          const isRouterOutput = newNode.data.officialName === "OutputRouter";

          if (
            (isRouterInput || isRouterOutput) &&
            !newNode.data.params?.routingPort
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

            let uniquePortName = DEFAULT_ROUTING_PORT;

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
                    uniquePortName = `${DEFAULT_ROUTING_PORT}-${counter}`;
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
              const existingPosition = existingYNode.get(
                "position",
              ) as Y.Map<unknown>;

              if (existingPosition) {
                existingPosition.set("x", change.position.x);
                existingPosition.set("y", change.position.y);
              } else {
                const newPosition = new Y.Map<unknown>();
                newPosition.set("x", change.position.x);
                newPosition.set("y", change.position.y);
                existingYNode.set("position", newPosition);
              }
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
              const existingMeasured = existingYNode.get(
                "measured",
              ) as Y.Map<unknown>;

              if (existingMeasured) {
                existingMeasured.set("width", change.dimensions.width);
                existingMeasured.set("height", change.dimensions.height);
              } else {
                const newMeasured = new Y.Map<unknown>();
                newMeasured.set("width", change.dimensions.width);
                newMeasured.set("height", change.dimensions.height);
                existingYNode?.set("measured", newMeasured);
              }

              if (change.setAttributes) {
                const existingStyle = existingYNode.get(
                  "style",
                ) as Y.Map<unknown>;

                if (existingStyle) {
                  existingStyle.set("width", change.dimensions.width + "px");
                  existingStyle.set("height", change.dimensions.height + "px");
                } else {
                  const newStyle = new Y.Map<unknown>();
                  newStyle.set("width", change.dimensions.width + "px");
                  newStyle.set("height", change.dimensions.height + "px");
                  existingYNode?.set("style", newStyle);
                }
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
        updatedParams?: any;
        updatedCustomizations?: any;
        isDisabled?: boolean;
      }[],
    ) =>
      undoTrackerActionWrapper(() => {
        const yNodes = currentYWorkflow?.get("nodes") as YNodesMap | undefined;
        if (!yNodes) return;

        const nodes = Object.values(yNodes.toJSON()) as Node[];

        nodesToChange.forEach(
          ({ nodeId, updatedParams, updatedCustomizations, isDisabled }) => {
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
            if (isDisabled !== undefined) yData?.set("isDisabled", isDisabled);
          },
        );
      }),
    [currentYWorkflow, rawWorkflows, yWorkflows, undoTrackerActionWrapper],
  );

  return {
    handleYNodesAdd,
    handleYNodesChange,
    handleYNodesDataUpdate,
  };
};
