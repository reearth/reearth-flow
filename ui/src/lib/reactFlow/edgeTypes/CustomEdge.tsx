import {
  BaseEdge,
  EdgeLabelRenderer,
  EdgeProps,
  getBezierPath,
} from "@xyflow/react";
import { memo } from "react";

import { Edge } from "@flow/types";

export type CustomEdgeProps = EdgeProps<Edge>;

const CustomEdge: React.FC<CustomEdgeProps> = ({
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
  const [edgePath, labelX, labelY] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  });

  // TODO: Implement when intermediate data becomes available @KaWaite

  const intermediateDataFeatureLength: number | undefined = undefined;
  return (
    <>
      <BaseEdge id={id} path={edgePath} markerEnd={markerEnd} />
      <EdgeLabelRenderer>
        {intermediateDataFeatureLength && (
          <div
            style={{
              position: "absolute",
              transform: `translate(-50%, -50%) translate(${labelX}px, ${labelY}px)`,
              pointerEvents: "all",
            }}
            className="nodrag nopan size-[12px] rounded bg-accent-foreground"
            onClick={() => console.log("I AM JUST A NUMBER")}>
            <p className="text-center align-middle text-[8px] text-black">
              {intermediateDataFeatureLength}
            </p>
          </div>
        )}
      </EdgeLabelRenderer>
      {/* <path
        d={edgePath}
        stroke="#27272A"
        strokeWidth="2"
        strokeDasharray="5,5"
        fill="none">
        <animate
          attributeName="stroke-dashoffset"
          from="10"
          to="0"
          dur="0.5s"
          repeatCount="indefinite"
        />
      </path> */}
      {/* <g>
        <circle className="opacity-25" r="8" fill="#752236">
          <animateMotion dur="4s" repeatCount="indefinite" path={edgePath} />
        </circle>
        <circle
          style={{ filter: `drop-shadow(3px 3px 5px #471a27)` }}
          r="4"
          fill="#752236">
          <animateMotion dur="4s" repeatCount="indefinite" path={edgePath} />
        </circle>
      </g> */}
    </>
  );
};

export default memo(CustomEdge);
