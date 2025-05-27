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

  const isSubworkflow = useMemo(() => {
    if (!currentYWorkflow) return false;
    const workflowId = currentYWorkflow.get("id")?.toJSON();
    return workflowId !== DEFAULT_ENTRY_GRAPH_ID;
  }, [currentYWorkflow]);

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

  const handleNodeSettings = useCallback(
    (_e: MouseEvent | undefined, nodeId: string) => {
      handleNodeLocking(nodeId);
    },
    [handleNodeLocking],
  );

  const { handleProjectExport } = useProjectExport(project);

  return {
    currentWorkflowId,
    isSubworkflow,
    nodes,
    edges,
    openWorkflows,
    isMainWorkflow,
    hoveredDetails,
    locallyLockedNode,
    handleProjectExport,
    handleNodeHover,
    handleNodesChange: handleYNodesChange,
    handleNodeSettings,
    handleEdgeHover,
    handleWorkflowOpen,
    handleWorkflowClose,
    handleCurrentWorkflowIdChange,
  };
};
