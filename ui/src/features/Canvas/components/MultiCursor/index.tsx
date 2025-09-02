import { useReactFlow, ViewportPortal } from "@xyflow/react";
import { throttle } from "lodash-es";
import { useCallback, useEffect, useMemo } from "react";
import { useUsers } from "y-presence";
import type { Awareness } from "y-protocols/awareness";

import { Cursor } from "./PerfectCursor";

type MultiCursorProps = {
  awareness: Awareness;
  currentUserName?: string;
  onCursorUpdate?: (
    updateFn: (clientX: number, clientY: number) => void,
  ) => void;
};

type UserData = {
  color?: string;
  cursor?: { x: number; y: number };
};

const MultiCursor: React.FC<MultiCursorProps> = ({
  awareness,
  onCursorUpdate,
}) => {
  const users = useUsers(awareness);
  const { screenToFlowPosition } = useReactFlow();
  useEffect(() => {
    if (awareness && !awareness.getLocalState()?.color) {
      const colors = [
        "#ef4444",
        "#f59e0b",
        "#10b981",
        "#3b82f6",
        "#8b5cf6",
        "#ec4899",
        "#06b6d4",
        "#84cc16",
      ];
      const color = colors[Math.floor(Math.random() * colors.length)];
      awareness.setLocalStateField("color", color);
      awareness.setLocalStateField("clientId", awareness.clientID);
    }
  }, [awareness]);

  const updateCursor = useCallback(
    (clientX: number, clientY: number) => {
      const flowPosition = screenToFlowPosition(
        {
          x: clientX,
          y: clientY,
        },
        { snapToGrid: false },
      );
      awareness.setLocalStateField("cursor", flowPosition);
    },
    [awareness, screenToFlowPosition],
  );
  const throttledUpdateCursor = useMemo(
    () => throttle(updateCursor, 16, { leading: true, trailing: true }),
    [updateCursor],
  );

  useEffect(() => {
    if (!onCursorUpdate) return;
    onCursorUpdate(throttledUpdateCursor);
    return () => throttledUpdateCursor.cancel();
  }, [onCursorUpdate, throttledUpdateCursor]);
  console.log("awareness", awareness.clientID);
  return (
    <ViewportPortal>
      {Array.from(users.entries() as IterableIterator<[number, UserData]>).map(
        ([key, value]) => {
          if (key === awareness.clientID) return null;
          if (!value.cursor) return null;

          return (
            <div
              key={key}
              style={{
                position: "absolute",
                left: value.cursor.x,
                top: value.cursor.y,
                pointerEvents: "none",
                zIndex: 1000,
              }}>
              <Cursor color={value.color} point={[0, 0]} />
            </div>
          );
        },
      )}
    </ViewportPortal>
  );
};

export default MultiCursor;
