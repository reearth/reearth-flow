import { useEffect, useRef, useState } from "react";
import { useY } from "react-yjs";
import { WebsocketProvider } from "y-websocket";
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
  const yWebSocketRef = useRef<WebsocketProvider | null>(null);
  useEffect(() => () => yWebSocketRef.current?.destroy(), []);

  const [{ yWorkflows }] = useState(() => {
    const yDoc = new Y.Doc();
    yWebSocketRef.current = new WebsocketProvider(
      "ws://localhost:8000",
      workflowId ? workflowId : "",
      yDoc,
      { params: { token: "nyaan" } },
    );

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

  const { currentYWorkflow, handleWorkflowAdd, handleWorkflowsRemove } =
    useYWorkflow({
      yWorkflows,
      workflows,
      currentWorkflowIndex,
      setWorkflows,
      setOpenWorkflowIds,
      handleWorkflowIdChange,
      handleWorkflowOpen,
    });

  const nodes = useY(
    currentYWorkflow?.get("nodes") ?? new Y.Array<Node>(),
  ) as Node[];
  const edges = useY(
    currentYWorkflow?.get("edges") ?? new Y.Array<Edge>(),
  ) as Edge[];

  const { handleNodesUpdate } = useYNode({
    currentYWorkflow,
    handleWorkflowsRemove,
  });

  const { handleEdgesUpdate } = useYEdge(currentYWorkflow);

  return {
    nodes,
    edges,
    openWorkflows,
    handleWorkflowClose,
    handleWorkflowAdd,
    handleNodesUpdate,
    handleEdgesUpdate,
  };
};
