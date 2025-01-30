import { Dispatch, SetStateAction, useCallback } from "react";
import * as Y from "yjs";

import type { Edge, Node, NodeChange } from "@flow/types";

import { fromYjsText, yNodeConstructor } from "../conversions";
import type { YNodesArray, YNodeValue, YWorkflow } from "../types";

import { cleanupPseudoPorts, updateParentYWorkflowNode } from "./utils";

export default ({
  currentYWorkflow,
  yWorkflows,
  rawWorkflows,
  setSelectedNodeIds,
  undoTrackerActionWrapper,
  handleWorkflowsRemove,
}: {
  currentYWorkflow: YWorkflow;
  yWorkflows: Y.Array<YWorkflow>;
  rawWorkflows: Record<string, string | Node[] | Edge[]>[];
  setSelectedNodeIds: Dispatch<SetStateAction<string[]>>;
  undoTrackerActionWrapper: (callback: () => void) => void;
  handleWorkflowsRemove: (workflowId: string[]) => void;
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

        // NOTE: if node is batch, we need to put it at the front
        // If its not a batch, we need to do useBatch stuff to
        // find if it becomes a batch's child
        yNodes.push(newYNodes);
      });
    },
    [currentYWorkflow, setSelectedNodeIds, undoTrackerActionWrapper],
  );

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

                  if (nodeToDelete.type === "subworkflow") {
                    handleWorkflowsRemove([change.id]);
                  } else if (nodeToDelete.data.params?.routingPort) {
                    const workflowIndex = rawWorkflows.findIndex((w) => {
                      const nodes = w.nodes as Node[];
                      return nodes.some(
                        (n) =>
                          n.id ===
                          (currentYWorkflow.get("id")?.toJSON() as string),
                      );
                    });
                    const yParentWorkflow = yWorkflows.get(workflowIndex);
                    if (yParentWorkflow) {
                      cleanupPseudoPorts(
                        currentYWorkflow.get("id")?.toJSON() as string,
                        yParentWorkflow,
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
      handleWorkflowsRemove,
      rawWorkflows,
      yWorkflows,
    ],
  );

  const handleYNodeParamsUpdate = useCallback(
    (nodeId: string, params: any) =>
      undoTrackerActionWrapper(() => {
        const yNodes = currentYWorkflow?.get("nodes") as
          | YNodesArray
          | undefined;
        if (!yNodes) return;

        const nodes = yNodes.toJSON() as Node[];

        const nodeIndex = nodes.findIndex((n) => n.id === nodeId);
        const node = nodes[nodeIndex];

        if (!node) return;

        // if params.routingPort exists, it is a subworkflow.
        if (params.routingPort) {
          const currentWorkflowId = currentYWorkflow
            .get("id")
            ?.toJSON() as string;

          const parentWorkflowIndex = rawWorkflows.findIndex((w) => {
            const nodes = w.nodes as Node[];
            return nodes.some((n) => n.id === currentWorkflowId);
          });
          const yParentWorkflow = yWorkflows.get(parentWorkflowIndex);

          // From here we are updating pseudoInputs and pseudoOutputs.
          // These only exist on subworkflow nodes.
          updateParentYWorkflowNode(
            currentWorkflowId,
            yParentWorkflow,
            node,
            params,
          );
        }

        const yData = yNodes.get(nodeIndex)?.get("data") as Y.Map<YNodeValue>;
        yData?.set("params", params);
      }),
    [currentYWorkflow, rawWorkflows, yWorkflows, undoTrackerActionWrapper],
  );

  return {
    handleYNodesAdd,
    handleYNodesChange,
    handleYNodeParamsUpdate,
  };
};
