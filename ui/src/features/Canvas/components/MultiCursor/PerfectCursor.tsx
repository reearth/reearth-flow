import * as React from "react";

import { usePerfectCursor } from "./hooks";

export const Cursor = React.memo(
  ({ point, color }: { point?: number[]; color?: string }) => {
    const rCursor = React.useRef<SVGSVGElement>(null);

    const animateCursor = React.useCallback((point: number[]) => {
      const elm = rCursor.current;
      if (!elm) return;
      elm.style.setProperty(
        "transform",
        `translate(${point[0]}px, ${point[1]}px)`,
      );
    }, []);

    const onPointMove = usePerfectCursor(animateCursor);

    if (point) {
      onPointMove(point);
    }

    if (!point || !color) return null;

    return (
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
    );
  },
);
