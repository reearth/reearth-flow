import { BaseEdge, EdgeLabelRenderer, EdgeProps, getBezierPath } from "reactflow";

import { EdgeData } from "@flow/types";

const CustomEdge: React.FC<EdgeProps<EdgeData>> = ({
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

export { CustomEdge };

export const edgeTypes = {
  default: CustomEdge,
};
