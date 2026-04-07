import { BaseEdge, EdgeProps, getBezierPath } from "@xyflow/react";
import { memo } from "react";

import { Edge } from "@flow/types";

import useHooks from "./hooks";

export type CustomEdgeProps = EdgeProps<Edge> & {
  currentWorkflowId?: string;
};

const DefaultEdge: React.FC<CustomEdgeProps> = ({
  id,
  sourceX,
  sourceY,
  sourcePosition,
  targetX,
  targetY,
  targetPosition,
  selected,
  currentWorkflowId,
  // markerEnd,
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

  const {
    // sourceNodeStatus,
    jobStatus,
  } = useHooks({
    currentWorkflowId,
  });

  return (
    <>
      <BaseEdge id={id} path={edgePath} />
      {jobStatus === "completed" &&
        (selected ? (
          <path
            className="stroke-success"
            d={edgePath}
            strokeWidth="2"
            fill="none"
            markerEnd="url(#arrow)"
          />
        ) : (
          <path
            d={edgePath}
            className="stroke-success/60"
            strokeWidth="1"
            fill="none"
            markerEnd="url(#arrow)"
          />
        ))}
      {jobStatus === "queued" &&
        (selected ? (
          <path d={edgePath} stroke="#27272A" fill="none" className="pulse" />
        ) : (
          <path
            d={edgePath}
            stroke="#27272A"
            fill="none"
            className="stroke-dashed"
          />
        ))}
      {jobStatus === "running" && (
        <>
          <path
            d={edgePath}
            stroke="#27272A"
            strokeDasharray="10,10"
            fill="none">
            <animate
              attributeName="stroke-dashoffset"
              from="40"
              to="0"
              dur="3s"
              repeatCount="indefinite"
            />
          </path>
          <g>
            <circle className="opacity-25" r="6" fill="#ffffff">
              <animateMotion
                dur="6s"
                repeatCount="indefinite"
                path={edgePath}
              />
            </circle>
            <circle
              className="opacity-75"
              style={{ filter: `drop-shadow(3px 3px 5px #471a27)` }}
              r="3"
              fill="#bbffff">
              <animateMotion
                dur="6s"
                repeatCount="indefinite"
                path={edgePath}
              />
            </circle>
          </g>
        </>
      )}
      {/* {sourceNodeStatus === "failed" && (
        <path d={edgePath} stroke="#fc4444" strokeWidth="1" fill="none" />
      )} */}
    </>
  );
};

export default memo(DefaultEdge);
