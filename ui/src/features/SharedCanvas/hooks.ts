import { useReactFlow } from "@xyflow/react";
import { MouseEvent, useCallback, useEffect, useMemo, useState } from "react";
import { useY } from "react-yjs";
import { Map as YMap } from "yjs";

import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { useProjectExport } from "@flow/hooks";
import { rebuildWorkflow } from "@flow/lib/yjs/conversions";
import { YWorkflow } from "@flow/lib/yjs/types";
import useWorkflowTabs from "@flow/lib/yjs/useWorkflowTabs";
import useYNode from "@flow/lib/yjs/useYNode";
import { Edge, Node, Project } from "@flow/types";

import useNodeLocker from "../Editor/useNodeLocker";
import useUIState from "../Editor/useUIState";

export default ({
  yWorkflows,
  project,
  undoTrackerActionWrapper,
}: {
  yWorkflows: YMap<YWorkflow>;
  project?: Project;
  undoTrackerActionWrapper: (
    callback: () => void,
    originPrepend?: string,
  ) => void;
}) => {
  const { fitView } = useReactFlow();

  const [selectedNodeIds, setSelectedNodeIds] = useState<string[]>([]);

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

  const { hoveredDetails, handleNodeHover, handleEdgeHover } = useUIState({});

  useEffect(() => {
    fitView({ padding: 0.5 });
  }, [fitView]);

  const { locallyLockedNode, handleNodeLocking } = useNodeLocker({
    nodes,
    selectedNodeIds,
    setSelectedNodeIds,
  });

  const handleNodeDoubleClick = useCallback(
    (_e: MouseEvent | undefined, nodeId: string, subworkflowId?: string) => {
      if (subworkflowId) {
        handleWorkflowOpen(subworkflowId);
      } else {
        fitView({
          nodes: [{ id: nodeId }],
          duration: 500,
          padding: 2,
        });
        handleNodeLocking(nodeId);
      }
    },
    [handleWorkflowOpen, fitView, handleNodeLocking],
  );

  const { handleProjectExport } = useProjectExport(project);

  return {
    currentWorkflowId,
    nodes,
    edges,
    openWorkflows,
    isMainWorkflow,
    hoveredDetails,
    locallyLockedNode,
    handleProjectExport,
    handleNodeHover,
    handleNodesChange: handleYNodesChange,
    handleNodeDoubleClick,
    handleEdgeHover,
    handleWorkflowOpen,
    handleWorkflowClose,
    handleCurrentWorkflowIdChange,
  };
};
