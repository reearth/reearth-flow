import { useReactFlow } from "@xyflow/react";
import { MouseEvent, useCallback, useEffect, useMemo, useState } from "react";
import { useY } from "react-yjs";
import { Array as YArray } from "yjs";

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
  yWorkflows: YArray<YWorkflow>;
  project?: Project;
  undoTrackerActionWrapper: (callback: () => void) => void;
}) => {
  const { fitView } = useReactFlow();

  const [selectedNodeIds, setSelectedNodeIds] = useState<string[]>([]);

  const rawWorkflows = yWorkflows.map((w) => rebuildWorkflow(w));

  const [currentWorkflowId, setCurrentWorkflowId] = useState(
    rawWorkflows.find((rw) => rw.isMain)?.id ?? rawWorkflows[0].id,
  );

  const currentYWorkflow = yWorkflows.get(
    rawWorkflows.findIndex((w) => w.id === currentWorkflowId) || 0,
  );

  const rawNodes = useY(
    (currentYWorkflow.get("nodes") ?? new YArray()) as YArray<Node>, // TODO: this is typed wrong. Need to fix @KaWaite
  ) as Node[];

  // Non-persistant state needs to be managed here
  const nodes = useMemo(
    () =>
      rawNodes.map((node) => ({
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

  const edges = useY(
    (currentYWorkflow.get("edges") ?? new YArray()) as YArray<Edge>, // TODO: this is typed wrong. Need to fix @KaWaite
  ) as Edge[];

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
    selectedNodeIds,
    nodes,
  });

  const handleNodeDoubleClick = useCallback(
    (_e: MouseEvent | undefined, node: Node) => {
      if (node.type === "subworkflow" && node.data.subworkflowId) {
        handleWorkflowOpen(node.data.subworkflowId);
      } else {
        fitView({
          nodes: [{ id: node.id }],
          duration: 500,
          padding: 2,
        });
        handleNodeLocking(node.id);
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
