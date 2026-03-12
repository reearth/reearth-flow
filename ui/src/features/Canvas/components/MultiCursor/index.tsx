import { ViewportPortal } from "@xyflow/react";

import type { AwarenessUser } from "@flow/types";

import { Cursor } from "./PerfectCursor";

type MultiCursorProps = {
  users: Record<string, AwarenessUser>;
  currentWorkflowId: string;
};

const getRect = (r: {
  startX: number;
  startY: number;
  currentX: number;
  currentY: number;
}) => ({
  x: Math.min(r.startX, r.currentX),
  y: Math.min(r.startY, r.currentY),
  width: Math.abs(r.currentX - r.startX),
  height: Math.abs(r.currentY - r.startY),
});

const MultiCursor: React.FC<MultiCursorProps> = ({
  users,
  currentWorkflowId,
}) => {
  return (
    <ViewportPortal>
      {Object.entries(users).map(([key, value]) => {
        if (value.currentWorkflowId !== currentWorkflowId) return null;

        const rect = value.selectionRect ? getRect(value.selectionRect) : null;
        return (
          <div key={key}>
            {value.cursor && (
              <div
                style={{
                  position: "absolute",
                  left: value.cursor.x,
                  top: value.cursor.y,
                  pointerEvents: "none",
                  zIndex: 2000,
                }}>
                <Cursor
                  color={value.color}
                  point={[0, 0]}
                  userName={value.userName}
                />
              </div>
            )}

            {rect && (
              <div
                style={{
                  position: "absolute",
                  left: rect.x,
                  top: rect.y,
                  width: rect.width,
                  height: rect.height,
                  pointerEvents: "none",
                  zIndex: 1999,
                  border: `1px solid ${value.color}`,
                  background: `${value.color}22`,
                }}
              />
            )}
          </div>
        );
      })}
    </ViewportPortal>
  );
};

export default MultiCursor;
