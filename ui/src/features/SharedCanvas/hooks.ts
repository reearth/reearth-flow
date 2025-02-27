import { useMemo, useState } from "react";
import { Array as YArray } from "yjs";

import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { rebuildWorkflow } from "@flow/lib/yjs/conversions";
import { YWorkflow } from "@flow/lib/yjs/types";
import useWorkflowTabs from "@flow/lib/yjs/useWorkflowTabs";

import useUIState from "../Editor/useUIState";

export default ({ yWorkflows }: { yWorkflows: YArray<YWorkflow> }) => {
  const [currentWorkflowId, setCurrentWorkflowId] = useState(
    DEFAULT_ENTRY_GRAPH_ID,
  );

  const rawWorkflows = yWorkflows.map((w) => rebuildWorkflow(w));

  const nodes = useMemo(
    () => rawWorkflows.find((w) => w.id === currentWorkflowId)?.nodes ?? [],
    [currentWorkflowId, rawWorkflows],
  );
  const edges = useMemo(
    () => rawWorkflows.find((w) => w.id === currentWorkflowId)?.edges ?? [],
    [currentWorkflowId, rawWorkflows],
  );

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

  return {
    currentWorkflowId,
    nodes,
    edges,
    openWorkflows,
    isMainWorkflow,
    hoveredDetails,
    handleNodeHover,
    handleEdgeHover,
    handleWorkflowOpen,
    handleWorkflowClose,
    handleCurrentWorkflowIdChange,
  };
};
