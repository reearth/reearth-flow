import { useReactFlow } from "@xyflow/react";
import { useMemo, useCallback } from "react";

import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { Workflow } from "@flow/types";

export type SearchNodeResult = {
  id: string;
  displayName: string; // customName || officialName
  officialName: string;
  customName?: string;
  workflowId: string;
  workflowName: string;
  isMainWorkflow: boolean;
  nodeType: string;
};

export const useSearchNodes = ({
  rawWorkflows,
  currentWorkflowId,
  onWorkflowOpen,
}: {
  rawWorkflows: Workflow[];
  currentWorkflowId: string;
  onWorkflowOpen: (id: string) => void;
}) => {
  const { setCenter, getNode } = useReactFlow();

  const allNodes: SearchNodeResult[] = useMemo(() => {
    return rawWorkflows.flatMap((workflow) =>
      (workflow.nodes || []).map((node) => ({
        id: node.id,
        displayName:
          node.data.customizations?.customName || node.data.officialName,
        officialName: node.data.officialName,
        customName: node.data.customizations?.customName,
        workflowId: workflow.id,
        workflowName: workflow.name || "Unnamed Workflow",
        isMainWorkflow: workflow.id === DEFAULT_ENTRY_GRAPH_ID,
        nodeType: node.type || "default",
      })),
    );
  }, [rawWorkflows]);

  const handleNavigateToNode = useCallback(
    (node: SearchNodeResult) => {
      if (node.workflowId !== currentWorkflowId) {
        onWorkflowOpen(node.workflowId);
      }
      setTimeout(
        () => {
          const reactFlowNode = getNode(node.id);
          if (reactFlowNode) {
            setCenter(
              reactFlowNode.position.x + (reactFlowNode.width ?? 0) / 2,
              reactFlowNode.position.y + (reactFlowNode.height ?? 0) / 2,
              { zoom: 1.1, duration: 300 },
            );
          }
        },
        node.workflowId !== currentWorkflowId ? 100 : 0,
      );
    },
    [currentWorkflowId, onWorkflowOpen, getNode, setCenter],
  );

  return {
    allNodes,
    handleNavigateToNode,
  };
};
