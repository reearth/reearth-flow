import { OnConnect, OnEdgesChange, addEdge, applyEdgeChanges } from "@xyflow/react";
import { Dispatch, SetStateAction, useCallback } from "react";

import { Edge } from "@flow/types";

type Props = {
  setEdges: Dispatch<SetStateAction<Edge[]>>;
};

export default ({ setEdges }: Props) => {
  const handleEdgesChange: OnEdgesChange = useCallback(
    changes => setEdges(eds => applyEdgeChanges(changes, eds)),
    [setEdges],
  );

  const handleConnect: OnConnect = useCallback(
    connection => setEdges(eds => addEdge(connection, eds)),
    [setEdges],
  );

  return {
    handleEdgesChange,
    handleConnect,
  };
};
