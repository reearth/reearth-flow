import { Dispatch, SetStateAction, useCallback } from "react";
import * as Y from "yjs";

import type { Edge, Node, NodeChange } from "@flow/types";

import { fromYjsText, yNodeConstructor } from "./conversions";
import type { YNodesArray, YNodeValue, YWorkflow } from "./types";
import { updateParentYWorkflow } from "./useParentYWorkflow";
import { removeParentYWorkflowNodePseudoPort } from "./useParentYWorkflow/removeParentYWorkflowNodePseudoPort";

export default ({
  currentYWorkflow,
  yWorkflows,
  rawWorkflows,
  setSelectedNodeIds,
  undoTrackerActionWrapper,
  handleYWorkflowsRemove,
}: {
  currentYWorkflow: YWorkflow;
  yWorkflows: Y.Array<YWorkflow>;
  rawWorkflows: Record<string, string | Node[] | Edge[]>[];
  setSelectedNodeIds: Dispatch<SetStateAction<string[]>>;
  undoTrackerActionWrapper: (callback: () => void) => void;
  handleYWorkflowsRemove?: (workflowId: string[]) => void;
}) => {
  const handleYNodesAdd = useCallback(
    (newNodes: Node[]) => {
      undoTrackerActionWrapper(() => {
        const yNodes = currentYWorkflow.get("nodes") as YNodesArray | undefined;
        if (!yNodes) return;
        const newYNodes = newNodes.map((newNode) => yNodeConstructor(newNode));

        newNodes.forEach((newNode) => {
          if (newNode.selected) {
            setSelectedNodeIds((snids) => {
              return [...snids, newNode.id];
            });
          }
        });

        // If any new nodes are batches, we need to put it at the front
        const insertIndex = newNodes.some((n) => n.type === "batch")
          ? 0
          : yNodes.length;

        yNodes.insert(insertIndex, newYNodes);
      });
    },
    [currentYWorkflow, setSelectedNodeIds, undoTrackerActionWrapper],
  );

  // This is based off of react-flow node changes, which includes removal
  // but not addtion. This is why we have a separate function for adding nodes.
  const handleYNodesChange = useCallback(
    (changes: NodeChange[]) => {
      const yNodes = currentYWorkflow?.get("nodes") as YNodesArray | undefined;
      if (!yNodes) return;

      const existingNodesMap = new Map(
        Array.from(yNodes).map((yNode, index) => [
          yNode.get("id")?.toString(),
          { yNode, index },
        ]),
      );

      undoTrackerActionWrapper(() => {
        changes.forEach((change) => {
          switch (change.type) {
            case "position": {
              const existing = existingNodesMap.get(change.id);

              if (existing && change.position) {
                const newPosition = new Y.Map<unknown>();
                newPosition.set("x", change.position.x);
                newPosition.set("y", change.position.y);
                existing?.yNode.set("position", newPosition);
              }
              break;
            }
            case "replace": {
              const existing = existingNodesMap.get(change.id);

              if (existing && change.item) {
                const newNode = yNodeConstructor(change.item);
                yNodes.delete(existing.index, 1);
                yNodes.insert(existing.index, [newNode]);
              }
              break;
            }
            case "dimensions": {
              const existing = existingNodesMap.get(change.id);

              if (existing && change.dimensions) {
                const newMeasured = new Y.Map<unknown>();
                newMeasured.set("width", change.dimensions.width);
                newMeasured.set("height", change.dimensions.height);
                existing?.yNode.set("measured", newMeasured);

                if (change.setAttributes) {
                  const newStyle = new Y.Map<unknown>();
                  newStyle.set("width", change.dimensions.width + "px");
                  newStyle.set("height", change.dimensions.height + "px");
                  existing?.yNode.set("style", newStyle);
                }
              }
              break;
            }
            case "remove": {
              const existing = existingNodesMap.get(change.id);

              if (existing) {
                const index = Array.from(yNodes).findIndex(
                  (yn) => fromYjsText(yn.get("id") as Y.Text) === change.id,
                );

                if (index !== -1) {
                  const nodeToDelete = Array.from(yNodes)[
                    index
                  ].toJSON() as Node;

                  if (
                    nodeToDelete.type === "subworkflow" &&
                    nodeToDelete.data.subworkflowId
                  ) {
                    handleYWorkflowsRemove?.([nodeToDelete.data.subworkflowId]);
                  } else if (nodeToDelete.data.params?.routingPort) {
                    const workflowIndex = rawWorkflows.findIndex((w) => {
                      const nodes = w.nodes as Node[];
                      return nodes.some(
                        (n) =>
                          n.id ===
                          (currentYWorkflow.get("id")?.toJSON() as string),
                      );
                    });
                    const parentYWorkflow = yWorkflows.get(workflowIndex);
                    if (parentYWorkflow) {
                      removeParentYWorkflowNodePseudoPort(
                        currentYWorkflow.get("id")?.toJSON() as string,
                        parentYWorkflow,
                        nodeToDelete,
                      );
                    }
                  }

                  setSelectedNodeIds((snids) => {
                    return snids.filter((snid) => snid !== change.id);
                  });

                  yNodes.delete(index, 1);
                }
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
    },
    [
      currentYWorkflow,
      setSelectedNodeIds,
      undoTrackerActionWrapper,
      handleYWorkflowsRemove,
      rawWorkflows,
      yWorkflows,
    ],
  );

  const handleYNodeParamsUpdate = useCallback(
    (nodeId: string, newParams: any) =>
      undoTrackerActionWrapper(() => {
        const yNodes = currentYWorkflow?.get("nodes") as
          | YNodesArray
          | undefined;
        if (!yNodes) return;

        const nodes = yNodes.toJSON() as Node[];

        const nodeIndex = nodes.findIndex((n) => n.id === nodeId);
        const prevNode = nodes[nodeIndex];

        if (!prevNode) return;

        // if params.routingPort exists, it's parent is a subworkflow and
        // we need to update pseudoInputs and pseudoOutputs on the parent node.
        if (newParams.routingPort) {
          const currentWorkflowId = currentYWorkflow
            .get("id")
            ?.toJSON() as string;

          const parentWorkflowIndex = rawWorkflows.findIndex((w) => {
            const nodes = w.nodes as Node[];
            return nodes.some(
              (n) => n.data.subworkflowId === currentWorkflowId,
            );
          });
          const parentYWorkflow = yWorkflows.get(parentWorkflowIndex);

          updateParentYWorkflow(
            currentWorkflowId,
            parentYWorkflow,
            prevNode,
            newParams,
          );
        }

        const yData = yNodes.get(nodeIndex)?.get("data") as Y.Map<YNodeValue>;
        yData?.set("params", newParams);
      }),
    [currentYWorkflow, rawWorkflows, yWorkflows, undoTrackerActionWrapper],
  );

  return {
    handleYNodesAdd,
    handleYNodesChange,
    handleYNodeParamsUpdate,
  };
};
