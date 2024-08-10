import { useEffect, useRef, useState } from "react";
import { useY } from "react-yjs";
import * as Y from "yjs";

import type { Edge, Node } from "@flow/types";

import { WebsocketProvider } from 'y-websocket'
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
  const yDocRef = useRef<Y.Doc | null>(null)
  const yWebSocketRef = useRef<WebsocketProvider | null>(null);

  const [{ yWorkflows }, setYWorkflows] = useState<{ yWorkflows: Y.Array<YWorkflow> }>({ yWorkflows: new Y.Array() });


  useEffect(() => {
    yDocRef.current = new Y.Doc()
    yWebSocketRef.current = new WebsocketProvider(
      "ws://localhost:8000",
      "test room",
      yDocRef.current,
      { params: { token: "nyaan" } },
    )
    const yWorkflows = yDocRef.current.getArray<YWorkflow>("workflows");
    const yWorkflow = yWorkflowBuilder("main", "Main Workflow");
    yWorkflows.push([yWorkflow]);
    setYWorkflows({ yWorkflows });

    return () => {
      yDocRef.current?.destroy();
      yWebSocketRef.current?.destroy();
    }
  }, [])

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
