import {
  OnConnect,
  OnEdgesChange,
  OnReconnect,
  addEdge,
  applyEdgeChanges,
  reconnectEdge,
} from "@xyflow/react";
import { useCallback } from "react";

import { Edge } from "@flow/types";

type Props = {
  edges: Edge[];
  onEdgeSelection: (idsToAdd: string[], idsToDelete: string[]) => void;
  onEdgeChange: (edges: Edge[]) => void;
};

export default ({ edges, onEdgeSelection, onEdgeChange }: Props) => {
  const handleEdgesChange: OnEdgesChange = useCallback(
    (changes) => {
      const idsToAdd: string[] = [];
      const idsToDelete: string[] = [];

      changes.forEach((c) => {
        if (c.type === "select") {
          if (c.selected) {
            idsToAdd.push(c.id);
          } else if (c.selected === false) {
            idsToDelete.push(c.id);
          }
        }
      });
      onEdgeSelection(idsToAdd, idsToDelete);

      onEdgeChange(applyEdgeChanges(changes, edges));
    },
    [edges, onEdgeSelection, onEdgeChange],
  );

  const handleConnect: OnConnect = useCallback(
    (connection) => onEdgeChange(addEdge(connection, edges)),
    [edges, onEdgeChange],
  );

  const handleReconnect: OnReconnect = useCallback(
    (oldEdge, newConnection) =>
      onEdgeChange(reconnectEdge(oldEdge, newConnection, edges)),
    [edges, onEdgeChange],
  );

  return {
    handleEdgesChange,
    handleConnect,
    handleReconnect,
  };
};
