import { getStraightPath, useStore } from "@xyflow/react";

import { AwarenessUser } from "@flow/types";

type GhostEdgeProps = {
  user: AwarenessUser;
};

const GhostEdge: React.FC<GhostEdgeProps> = ({ user }) => {
  const nodeLookup = useStore((s) => s.nodeLookup);

  if (!user.draggingEdge || !user.cursor) return null;

  const { nodeId, handleId, handleType } = user.draggingEdge;
  const internalNode = nodeLookup.get(nodeId);
  if (!internalNode) return null;

  const isSource = handleType !== "target";
  const handleBounds = internalNode.internals.handleBounds;
  const handles = isSource ? handleBounds?.source : handleBounds?.target;
  const handle = handleId
    ? handles?.find((h) => h.id === handleId)
    : handles?.[0];

  let handleX: number;
  let handleY: number;

  if (handle) {
    handleX =
      internalNode.internals.positionAbsolute.x + handle.x + handle.width / 2;
    handleY =
      internalNode.internals.positionAbsolute.y + handle.y + handle.height / 2;
  } else {
    // Fallback: right/left edge at vertical center
    const w = internalNode.measured?.width ?? 0;
    const h = internalNode.measured?.height ?? 0;
    handleX = isSource
      ? internalNode.internals.positionAbsolute.x + w
      : internalNode.internals.positionAbsolute.x;
    handleY = internalNode.internals.positionAbsolute.y + h / 2;
  }

  const [edgePath] = getStraightPath({
    sourceX: isSource ? handleX : user.cursor.x,
    sourceY: isSource ? handleY : user.cursor.y,
    targetX: isSource ? user.cursor.x : handleX,
    targetY: isSource ? user.cursor.y : handleY,
  });

  return (
    <svg
      className="pointer-events-none absolute top-0 left-0 overflow-visible"
      style={{
        zIndex: 2001,
      }}>
      <path
        d={edgePath}
        stroke={user.color}
        strokeWidth={2}
        fill="none"
        opacity={0.8}
      />
    </svg>
  );
};

export default GhostEdge;
