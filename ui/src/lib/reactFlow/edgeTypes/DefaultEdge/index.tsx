import { Table, X } from "@phosphor-icons/react";
import {
  BaseEdge,
  EdgeLabelRenderer,
  EdgeProps,
  getBezierPath,
} from "@xyflow/react";

import { Edge } from "@flow/types";

import useHooks from "./hooks";

export type CustomEdgeProps = EdgeProps<Edge>;

const DefaultEdge: React.FC<CustomEdgeProps> = ({
  id,
  sourceX,
  sourceY,
  sourcePosition,
  targetX,
  targetY,
  targetPosition,
  selected,
  // markerEnd,
  // ...props
}) => {
  const [edgePath, labelX, labelY] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  });

  const { edgeStatus, intermediateDataUrl, handleIntermediateDataSet } =
    useHooks({
      id,
      selected,
    });

  return (
    <>
      <BaseEdge id={id} path={edgePath} />
      <EdgeLabelRenderer>
        {edgeStatus === "failed" ? (
          <X
            className="nodrag nopan absolute size-[20px] origin-center rounded-full border border-destructive bg-primary fill-destructive p-1"
            weight="bold"
            style={{
              pointerEvents: "all",
              transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`,
            }}
          />
        ) : edgeStatus === "completed" && intermediateDataUrl ? (
          <Table
            className="nodrag nopan absolute size-[30px] origin-center rounded-full border border-white bg-primary p-1 transition-[height,width] hover:size-[50px]"
            style={{
              pointerEvents: "all",
              transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`,
            }}
            weight="bold"
            onDoubleClick={handleIntermediateDataSet}
          />
        ) : null}
      </EdgeLabelRenderer>
      {edgeStatus === "completed" && (
        <path
          d={edgePath}
          stroke="#00a340"
          strokeWidth="1"
          fill="none"
          markerEnd="url(#arrow)"
        />
      )}
      {edgeStatus === "inProgress" && (
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
      {edgeStatus === "failed" && (
        <path d={edgePath} stroke="#fc4444" strokeWidth="1" fill="none" />
      )}
    </>
  );
};

export default DefaultEdge;
