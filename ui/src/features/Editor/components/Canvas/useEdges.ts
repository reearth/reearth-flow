import { OnConnect, OnEdgesChange, addEdge, applyEdgeChanges } from "@xyflow/react";
import { Dispatch, SetStateAction } from "react";

import { Edge } from "@flow/types";

type Props = {
  setEdges: Dispatch<SetStateAction<Edge[]>>;
};

export default ({ setEdges }: Props) => {
  const handleEdgesChange: OnEdgesChange = changes =>
    setEdges(eds => applyEdgeChanges(changes, eds));

  const handleConnect: OnConnect = connection => setEdges(eds => addEdge(connection, eds));

  return {
    handleEdgesChange,
    handleConnect,
  };
};
