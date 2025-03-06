import { ConnectionLineComponent, getStraightPath } from "@xyflow/react";
import { memo } from "react";

export const connectionLineStyle = {
  strokeWidth: 1,
  stroke: "#a1a1aa",
};

const CustomConnectionLine: ConnectionLineComponent = ({
  fromX,
  fromY,
  toX,
  toY,
  connectionLineStyle,
}) => {
  const [edgePath] = getStraightPath({
    sourceX: fromX,
    sourceY: fromY,
    targetX: toX,
    targetY: toY,
  });

  return (
    <g>
      <path style={connectionLineStyle} d={edgePath} />
      <circle cx={toX} cy={toY} fill="#a1a1aa" r={2} />
    </g>
  );
};

export default memo(CustomConnectionLine);
