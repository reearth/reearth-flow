import { useState } from "react";
import * as Y from "yjs";

import { fromYjsArray } from "@flow/lib/yjs/conversions";
import { Edge, Node } from "@flow/types";

export default () => {
  const [{ yNodes, yEdges }] = useState(() => {
    const yDoc = new Y.Doc();
    return { yDoc, yNodes: yDoc.getArray("nodes"), yEdges: yDoc.getArray("edges") };
  });

  const nodes = fromYjsArray(yNodes);
  const edges = fromYjsArray(yEdges);

  const handleYjsNodesUpdate = (newNodes: Node[]) => {
    yNodes.delete(0, yNodes.length);
    yNodes.insert(0, newNodes);
  };

  const handleYjsEdgesUpdate = (newEdges: Edge[]) => {
    yEdges.delete(0, yEdges.length);
    yEdges.insert(0, newEdges);
  };

  return { nodes, edges, handleYjsNodesUpdate, handleYjsEdgesUpdate };
};
