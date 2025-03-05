import { BaseEdge, EdgeProps, getBezierPath } from "@xyflow/react";
import { memo } from "react";

import { Edge } from "@flow/types";

export type CustomEdgeProps = EdgeProps<Edge>;

const DefaultEdge: React.FC<CustomEdgeProps> = ({
  id,
  sourceX,
  sourceY,
  sourcePosition,
  targetX,
  targetY,
  targetPosition,
  markerEnd,
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

  // TODO: pass node status of source node
  // const nodeRunning = Math.random() < 0.5;

  return (
    <>
      <BaseEdge id={id} path={edgePath} markerEnd={markerEnd} />
      {/* {nodeRunning && (
        <>
          <path
            d={edgePath}
            stroke="#27272A"
            strokeWidth="2"
            strokeDasharray="20,20"
            fill="none">
            <animate
              attributeName="stroke-dashoffset"
              from="40"
              to="0"
              dur="1s"
              repeatCount="indefinite"
            />
          </path>
          <g>
            <circle className="opacity-25" r="8" fill="#ffffff">
              <animateMotion
                dur="5s"
                repeatCount="indefinite"
                path={edgePath}
              />
            </circle>
            <circle
              style={{ filter: `drop-shadow(3px 3px 5px #471a27)` }}
              r="3"
              fill="#bbffff">
              <animateMotion
                dur="5s"
                repeatCount="indefinite"
                path={edgePath}
              />
            </circle>
          </g>
        </>
      )} */}
    </>
  );
};

export default memo(DefaultEdge);
