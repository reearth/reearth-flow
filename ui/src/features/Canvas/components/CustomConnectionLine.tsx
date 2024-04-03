import { ConnectionLineComponent, getStraightPath } from "reactflow";

export const connectionLineStyle = {
  strokeWidth: 2,
  stroke: "white",
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
      <path style={connectionLineStyle} fill="green" d={edgePath} />
      <circle cx={toX} cy={toY} fill="green" r={5} stroke="black" strokeWidth={4.5} />
    </g>
  );
};

export { CustomConnectionLine };
