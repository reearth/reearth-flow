import { useStore } from "@xyflow/react";

import type { AwarenessUser } from "@flow/types";

type NodeSelectionProps = {
  user: AwarenessUser;
};

const NodeSelection: React.FC<NodeSelectionProps> = ({ user }) => {
  const nodeLookup = useStore((s) => s.nodeLookup);

  if (!user.selectedNodeIds?.length) return null;

  return (
    <>
      {user.selectedNodeIds.map((nodeId) => {
        const internalNode = nodeLookup.get(nodeId);
        if (!internalNode) return null;

        const { x, y } = internalNode.internals.positionAbsolute;
        const w = internalNode.measured?.width ?? 0;
        const h = internalNode.measured?.height ?? 0;

        return (
          <div
            key={nodeId}
            className="pointer-events-none absolute z-50 rounded-[6px]"
            style={{
              left: x,
              top: y,
              width: w,
              height: h,
              border: `2px solid ${user.color}`,
            }}
          />
        );
      })}
    </>
  );
};

export default NodeSelection;
