import { memo } from "react";

type CursorProps = {
  x: number;
  y: number;
  color: string;
  name: string;
};

const CursorComponent: React.FC<CursorProps> = ({ x, y, color, name }) => {
  return (
    <div
      className="pointer-events-none absolute z-50"
      style={{
        transform: `translate(${x}px, ${y}px)`,
        transition: "transform 0.15s ease-out",
      }}>
      {/* Cursor SVG */}
      <svg
        width="24"
        height="24"
        viewBox="0 0 24 24"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        style={{ filter: "drop-shadow(0 2px 4px rgba(0,0,0,0.2))" }}>
        <path
          d="M5.65376 12.3673H5.46026L5.31717 12.4976L0.500002 16.8829L0.500002 1.19841L11.7841 12.3673H5.65376Z"
          fill={color}
          stroke="white"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
      {/* User name label */}
      <div
        className="absolute top-3 left-5 rounded px-1.5 py-0.5 text-xs font-medium whitespace-nowrap text-white"
        style={{
          backgroundColor: color,
          boxShadow: "0 2px 4px rgba(0,0,0,0.2)",
        }}>
        {name}
      </div>
    </div>
  );
};

export default memo(CursorComponent);
