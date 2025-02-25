import {
  EdgeChange,
  OnConnect,
  OnEdgesChange,
  OnReconnect,
} from "@xyflow/react";
import { useCallback } from "react";

import { Edge } from "@flow/types";

type Props = {
  edges: Edge[];
  onEdgesAdd: (newEdges: Edge[]) => void;
  onEdgesChange: (changes: EdgeChange[]) => void;
};

export default ({ edges, onEdgesAdd, onEdgesChange }: Props) => {
  const handleEdgesChange: OnEdgesChange<Edge> = useCallback(
    (changes) => onEdgesChange(changes),
    [onEdgesChange],
  );

  const handleConnect: OnConnect = useCallback(
    (connection) => {
      // Reference: https://github.com/xyflow/xyflow/blob/043c8120ace53b562c0b3ec8194ccdef64ccad0e/packages/system/src/utils/edges/general.ts#L79
      const edgeId = `xy-edge__${connection.source}${connection.sourceHandle || ""}-${connection.target}${connection.targetHandle || ""}`;
      if (edges.find((e) => e.id === edgeId)) return;
      onEdgesAdd([
        {
          id: edgeId,
          ...connection,
        },
      ]);
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
      onEdgesChange([{ id: oldEdge.id, type: "replace", item: updatedEdge }]);
    },
    [onEdgesChange],
  );

  return {
    handleEdgesChange,
    handleConnect,
    handleReconnect,
  };
};
