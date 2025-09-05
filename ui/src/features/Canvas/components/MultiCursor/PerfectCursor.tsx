import { useRef, useCallback, useEffect, memo } from "react";

import { usePerfectCursor } from "./hooks";

export const Cursor = memo(
  ({
    point,
    color,
    userName,
  }: {
    point?: number[];
    color?: string;
    userName?: string;
  }) => {
    const rCursor = useRef<SVGSVGElement>(null);

    const animateCursor = useCallback((point: number[]) => {
      const elm = rCursor.current;
      if (!elm) return;
      elm.style.setProperty(
        "transform",
        `translate(${point[0]}px, ${point[1]}px)`,
      );
    }, []);

    const onPointMove = usePerfectCursor(animateCursor);

    useEffect(() => {
      if (point) {
        onPointMove(point);
      }
    }, [point, onPointMove]);

    if (!point || !color) return null;

    return (
      <>
        <svg
          ref={rCursor}
          className="cursor pointer-events-none"
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 35 35"
          width="35"
          height="35"
          fill="none"
          fillRule="evenodd"
          style={{
            pointerEvents: "none",
            position: "absolute",
            top: "-15px",
            left: "-15px",
          }}>
          <g fill="rgba(0,0,0,.2)" transform="translate(1,1)">
            <path d="m12 24.4219v-16.015l11.591 11.619h-6.781l-.411.124z" />
            <path d="m21.0845 25.0962-3.605 1.535-4.682-11.089 3.686-1.553z" />
          </g>
          <g fill="white">
            <path d="m12 24.4219v-16.015l11.591 11.619h-6.781l-.411.124z" />
            <path d="m21.0845 25.0962-3.605 1.535-4.682-11.089 3.686-1.553z" />
          </g>
          <g fill={color}>
            <path d="m19.751 24.4155-1.844.774-3.1-7.374 1.841-.775z" />
            <path d="m13 10.814v11.188l2.969-2.866.428-.139h4.768z" />
          </g>
        </svg>
        {userName && (
          <div
            className="absolute top-3 left-3 max-w-[150px] shrink-0 rounded px-1.5 py-0.5 text-xs font-medium whitespace-nowrap text-white"
            style={{
              backgroundColor: color,
              boxShadow: "0 2px 4px rgba(0,0,0,0.2)",
            }}>
            <p className="truncate">{userName}</p>
          </div>
        )}
      </>
    );
  },
);
