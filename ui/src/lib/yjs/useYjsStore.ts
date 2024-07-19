import { useState } from "react";
import { useY } from "react-yjs";
import * as Y from "yjs";

import { Edge, Node } from "@flow/types";

export default () => {
  const [{ yNodes, yEdges }] = useState(() => {
    const yDoc = new Y.Doc();
    const yNodes = yDoc.getArray<Node>("nodes");
    const yEdges = yDoc.getArray<Edge>("edges");
    return { yNodes, yEdges };
  });

  const nodes = useY(yNodes);
  const edges = useY(yEdges);

  const handleNodesUpdate = (newNodes: Node[]) => {
    yNodes.delete(0, yNodes.length);
    yNodes.insert(0, newNodes);
  };

  const handleEdgesUpdate = (newEdges: Edge[]) => {
    yEdges.delete(0, yEdges.length);
    yEdges.insert(0, newEdges);
  };

  return { nodes, edges, handleNodesUpdate, handleEdgesUpdate };
};
