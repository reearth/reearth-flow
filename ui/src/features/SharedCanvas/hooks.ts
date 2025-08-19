import { useReactFlow } from "@xyflow/react";
import { MouseEvent, useCallback, useEffect, useMemo, useState } from "react";
import { useY } from "react-yjs";
import { Map as YMap } from "yjs";

import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { rebuildWorkflow } from "@flow/lib/yjs/conversions";
import { YWorkflow } from "@flow/lib/yjs/types";
import useWorkflowTabs from "@flow/lib/yjs/useWorkflowTabs";
import useYNode from "@flow/lib/yjs/useYNode";
import type { Edge, Node } from "@flow/types";

export default ({
  yWorkflows,
  undoTrackerActionWrapper,
}: {
  yWorkflows: YMap<YWorkflow>;
  undoTrackerActionWrapper: (
    callback: () => void,
    originPrepend?: string,
  ) => void;
}) => {
  const { fitView } = useReactFlow();

  const [selectedNodeIds, setSelectedNodeIds] = useState<string[]>([]);
  const [openNode, setOpenNode] = useState<Node | undefined>(undefined);

  const [currentWorkflowId, setCurrentWorkflowId] = useState(
    DEFAULT_ENTRY_GRAPH_ID,
  );

  const rawWorkflows = Array.from(yWorkflows.entries()).map(([, yw]) =>
    rebuildWorkflow(yw),
  );

  const currentYWorkflow = yWorkflows.get(currentWorkflowId);

  const rawNodes = useY(currentYWorkflow?.get("nodes") ?? new YMap()) as Record<
    string,
    Node
  >;

  // Non-persistant state needs to be managed here
  const nodes = useMemo(
    () =>
      Object.values(rawNodes).map((node) => ({
        ...node,
        selected:
          selectedNodeIds.includes(node.id) && !node.selected
            ? true
            : (node.selected ?? false),
      })),
    [rawNodes, selectedNodeIds],
  );

  const { handleYNodesChange } = useYNode({
    currentYWorkflow,
    rawWorkflows,
    yWorkflows,
    setSelectedNodeIds,
    undoTrackerActionWrapper,
  });

  const rawEdges = useY(currentYWorkflow?.get("edges") ?? new YMap()) as Record<
    string,
    Edge
  >;

  const edges = useMemo(() => Object.values(rawEdges), [rawEdges]);

  const {
    openWorkflows,
    isMainWorkflow,
    handleWorkflowOpen,
    handleWorkflowClose,
    handleCurrentWorkflowIdChange,
  } = useWorkflowTabs({
    currentWorkflowId,
    rawWorkflows,
    setCurrentWorkflowId,
  });

  useEffect(() => {
    fitView({ padding: 0.5 });
  }, [fitView]);

  const handleOpenNode = useCallback(
    (nodeId?: string) => {
      if (!nodeId) {
        setOpenNode(undefined);
      } else {
        setOpenNode((on) =>
          on?.id === nodeId ? undefined : nodes.find((n) => n.id === nodeId),
        );
      }
    },
    [nodes, setOpenNode],
  );

  const handleNodeSettings = useCallback(
    (_e: MouseEvent | undefined, nodeId: string) => {
      handleOpenNode(nodeId);
    },
    [handleOpenNode],
  );

  return {
    currentWorkflowId,
    nodes,
    edges,
    openWorkflows,
    isMainWorkflow,
    openNode,
    handleNodesChange: handleYNodesChange,
    handleOpenNode,
    handleNodeSettings,
    handleWorkflowOpen,
    handleWorkflowClose,
    handleCurrentWorkflowIdChange,
  };
};
