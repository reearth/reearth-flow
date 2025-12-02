import { BaseEdge, EdgeProps, getBezierPath } from "@xyflow/react";

import { Edge } from "@flow/types";

export type CustomEdgeProps = EdgeProps<Edge>;

const SimpleEdge: React.FC<CustomEdgeProps> = ({
  id,
  sourceX,
  sourceY,
  sourcePosition,
  targetX,
  targetY,
  targetPosition,
  // ...props
}) => {
  const [edgePath] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  });

  return <BaseEdge id={id} path={edgePath} />;
};

export default SimpleEdge;
