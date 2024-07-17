import { BaseEdge, EdgeLabelRenderer, EdgeProps, EdgeTypes, getBezierPath } from "@xyflow/react";
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
  ...props
}) => {
  const [edgePath, labelX, labelY] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  });

  return (
    <>
      <BaseEdge id={id} path={edgePath} {...props} />
      <EdgeLabelRenderer>
        <div
          style={{
            position: "absolute",
            transform: `translate(-50%, -50%) translate(${labelX}px, ${labelY}px)`,
            pointerEvents: "all",
          }}
          className="nodrag nopan bg-zinc-400 rounded h-[12px] w-[12px]"
          onClick={() => console.log("I AM JUST A NUMBER")}>
          <p className="text-[8px] text-black align-middle text-center">4</p>
        </div>
      </EdgeLabelRenderer>
    </>
  );
};

export default memo(CustomEdge);

export const edgeTypes: EdgeTypes = {
  default: CustomEdge,
};
