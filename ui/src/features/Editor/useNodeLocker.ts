import { useReactFlow } from "@xyflow/react";
import { useCallback, useState } from "react";

import type { Node } from "@flow/types";

export default ({
  handleNodesUpdate,
}: {
  handleNodesUpdate: (newNodes: Node[]) => void;
}) => {
  const { getNodes } = useReactFlow<Node>();

  // Will be used to keep track of all locked nodes, local and for other users (while collaborative editing)
  const [lockedNodeIds, setLockedNodeIds] = useState<string[]>([]);

  // Can have only one node locked at a time (locally)
  const [locallyLockedNode, setLocallyLockedNode] = useState<Node | undefined>(
    undefined,
  );

  // consider making a node context and supplying vars and functions like this to the nodes that way
  const handleNodeLocking = useCallback(
    (nodeId: string) => {
      handleNodesUpdate(
        getNodes().map((n) => {
          if (n.id === nodeId) {
            const newNode = {
              ...n,
              data: {
                ...n.data,
                locked: !n.data.locked,
              },
            };

            setLockedNodeIds((ids) => {
              if (ids.includes(newNode.id)) {
                return ids.filter((id) => id !== nodeId);
              }
              return [...ids, newNode.id];
            });

            setLocallyLockedNode((lln) =>
              lln?.id === newNode.id ? undefined : newNode,
            );

            return newNode;
          }
          return n;
        }),
      );
    },
    [getNodes, handleNodesUpdate],
  );

  return {
    lockedNodeIds,
    locallyLockedNode,
    handleNodeLocking,
  };
};
