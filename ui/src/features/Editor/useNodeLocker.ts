import { useCallback, useEffect, useState, useRef } from "react";

import type { Node } from "@flow/types";

export default ({
  nodes,
  selectedNodeIds,
}: {
  nodes: Node[];
  selectedNodeIds: string[];
}) => {
  // Will be used to keep track of all locked nodes, local and for other users (while collaborative editing)
  const [lockedNodeIds, setLockedNodeIds] = useState<string[]>([]);

  // Can have only one node locked at a time (locally)
  const [locallyLockedNode, setLocallyLockedNode] = useState<Node | undefined>(
    undefined,
  );
  const isLockingRef = useRef(false);

  // When a node is deselected on the canvas, we need to unlock it
  useEffect(() => {
    if (
      locallyLockedNode &&
      !selectedNodeIds.includes(locallyLockedNode.id) &&
      !isLockingRef.current
    ) {
      setLocallyLockedNode(undefined);
      setLockedNodeIds((lln) =>
        lln.filter((id) => id !== locallyLockedNode?.id),
      );
    }
  }, [selectedNodeIds, locallyLockedNode, lockedNodeIds]);

  const handleNodeLocking = useCallback(
    (nodeId: string) => {
      isLockingRef.current = true;

      setLockedNodeIds((ids) => {
        if (ids.includes(nodeId)) {
          return ids.filter((id) => id !== nodeId);
        }
        return [...ids, nodeId];
      });

      setLocallyLockedNode((lln) =>
        lln?.id === nodeId ? undefined : nodes.find((n) => n.id === nodeId),
      );
      // handleNodesChange([{ id: nodeId, type: "locking", locked: !!locked }]);
    },
    [nodes],
  );

  return {
    lockedNodeIds,
    locallyLockedNode,
    handleNodeLocking,
  };
};
