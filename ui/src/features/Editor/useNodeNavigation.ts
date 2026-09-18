import { useReactFlow } from "@xyflow/react";
import { useCallback } from "react";

import type { NodeChange, Workflow } from "@flow/types";

/** Matches the zoom the search panel settles on, so both reveals land alike. */
const REVEAL_ZOOM = 1.1;

/**
 * Opening another workflow remounts the canvas and fits the new graph in view,
 * so centring has to wait for that pass or it is immediately undone.
 */
const WORKFLOW_SWITCH_DELAY = 100;

/**
 * Reveals a node anywhere in the project: opens the workflow holding it,
 * centres the canvas on it and leaves it as the only selection.
 *
 * Callers know a node id and nothing else — a diagnostic row names the node
 * that reported it, not the workflow it lives in — so the lookup happens here.
 */
export default ({
  rawWorkflows,
  currentWorkflowId,
  onWorkflowOpen,
  onNodesChange,
}: {
  rawWorkflows: Workflow[];
  currentWorkflowId: string;
  onWorkflowOpen: (workflowId: string) => void;
  onNodesChange?: (changes: NodeChange[]) => void;
}) => {
  const { getNode, getNodes, setCenter } = useReactFlow();

  return useCallback(
    (nodeId: string) => {
      const workflow = rawWorkflows.find((w) =>
        w.nodes?.some((node) => node.id === nodeId),
      );
      // A node the project no longer holds — a diagnostic outlives the action
      // that reported it once the graph is edited mid-run.
      if (!workflow) return;

      const switchesWorkflow = workflow.id !== currentWorkflowId;
      if (switchesWorkflow) onWorkflowOpen(workflow.id);

      setTimeout(
        () => {
          const node = getNode(nodeId);
          if (!node) return;

          setCenter(
            node.position.x + (node.width ?? 0) / 2,
            node.position.y + (node.height ?? 0) / 2,
            { zoom: REVEAL_ZOOM, duration: 300 },
          );

          // Everything else is deselected in the same batch so the revealed
          // node is unambiguous — a stale selection elsewhere on the canvas
          // would still drive the action bar and the node settings.
          onNodesChange?.([
            ...getNodes()
              .filter((other) => other.selected && other.id !== nodeId)
              .map((other): NodeChange => ({
                type: "select",
                id: other.id,
                selected: false,
              })),
            { type: "select", id: nodeId, selected: true },
          ]);
        },
        switchesWorkflow ? WORKFLOW_SWITCH_DELAY : 0,
      );
    },
    [
      rawWorkflows,
      currentWorkflowId,
      onWorkflowOpen,
      onNodesChange,
      getNode,
      getNodes,
      setCenter,
    ],
  );
};
