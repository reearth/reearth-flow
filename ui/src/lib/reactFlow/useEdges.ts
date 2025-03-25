import {
  EdgeChange,
  OnConnect,
  OnEdgesChange,
  OnReconnect,
} from "@xyflow/react";
import { useCallback } from "react";

import { Edge } from "@flow/types";
import { generateUUID } from "@flow/utils";

type Props = {
  edges: Edge[];
  onEdgesAdd?: (newEdges: Edge[]) => void;
  onEdgesChange?: (changes: EdgeChange[]) => void;
};

export default ({ edges, onEdgesAdd, onEdgesChange }: Props) => {
  const handleEdgesChange: OnEdgesChange<Edge> = useCallback(
    (changes) => onEdgesChange?.(changes),
    [onEdgesChange],
  );

  const handleConnect: OnConnect = useCallback(
    (connection) => {
      const newEdge: Edge = {
        id: generateUUID(),
        ...connection,
      };
      if (edges.find((e) => e.id === newEdge.id)) return;
      onEdgesAdd?.([newEdge]);
    },
    [edges, onEdgesAdd],
  );

  const handleReconnect: OnReconnect = useCallback(
    (oldEdge, newConnection) => {
      const updatedEdge = {
        ...oldEdge,
        source: newConnection.source,
        target: newConnection.target,
        sourceHandle: newConnection.sourceHandle,
        targetHandle: newConnection.targetHandle,
      };
      onEdgesChange?.([{ id: oldEdge.id, type: "replace", item: updatedEdge }]);
    },
    [onEdgesChange],
  );

  return {
    handleEdgesChange,
    handleConnect,
    handleReconnect,
  };
};
