import { useState } from "react";
import { useY } from "react-yjs";
import * as Y from "yjs";

import type { Edge, Node } from "@flow/types";

import useWorkflowTabs from "./useWorkflowTabs";
import useYEdge from "./useYEdge";
import useYNode from "./useYNode";
import useYWorkflow from "./useYWorkflow";
import { yWorkflowBuilder, type YWorkflow } from "./workflowBuilder";

export default ({
  workflowId,
  handleWorkflowIdChange,
}: {
  workflowId?: string;
  handleWorkflowIdChange: (id?: string) => void;
}) => {
  const [{ yWorkflows }] = useState(() => {
    // TODO: setup middleware/websocket provider
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
    const yWorkflow = yWorkflowBuilder("main", "Main Workflow");
    yWorkflows.push([yWorkflow]);

    return { yWorkflows };
  });

  const rawWorkflows = useY(yWorkflows);

  const {
    workflows,
    openWorkflows,
    currentWorkflowIndex,
    setWorkflows,
    setOpenWorkflowIds,
    handleWorkflowOpen,
    handleWorkflowClose,
  } = useWorkflowTabs({ workflowId, rawWorkflows, handleWorkflowIdChange });

  const { currentYWorkflow, handleWorkflowAdd, handleWorkflowRemove } = useYWorkflow({
    yWorkflows,
    workflows,
    currentWorkflowIndex,
    setWorkflows,
    setOpenWorkflowIds,
    handleWorkflowIdChange,
    handleWorkflowOpen,
  });

  const nodes = useY(currentYWorkflow?.get("nodes") ?? new Y.Array<Node>()) as Node[];
  const edges = useY(currentYWorkflow?.get("edges") ?? new Y.Array<Edge>()) as Edge[];

  const { handleNodesUpdate } = useYNode(currentYWorkflow);

  const { handleEdgesUpdate } = useYEdge(currentYWorkflow);

  return {
    nodes,
    edges,
    openWorkflows,
    handleWorkflowClose,
    handleWorkflowAdd,
    handleWorkflowRemove,
    handleNodesUpdate,
    handleEdgesUpdate,
  };
};
