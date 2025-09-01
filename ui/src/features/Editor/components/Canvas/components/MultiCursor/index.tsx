import { useReactFlow, ViewportPortal } from "@xyflow/react";
import { useCallback, useEffect, useRef } from "react";
import { useUsers } from "y-presence";

import { Cursor } from "./PerfectCursor";

type MultiCursorProps = {
  yDoc?: any;
  awareness: any;
  currentUserName?: string;
  onCursorUpdate?: (
    updateFn: (clientX: number, clientY: number) => void,
  ) => void;
};

const MultiCursor: React.FC<MultiCursorProps> = ({
  awareness,
  onCursorUpdate,
}) => {
  const users = useUsers(awareness, (state: any) => state);
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
    }
  }, [awareness]);

  const lastUpdateRef = useRef(0);
  const updateCursor = useCallback(
    (clientX: number, clientY: number) => {
      const now = Date.now();

      const flowPosition = screenToFlowPosition(
        {
          x: clientX,
          y: clientY,
        },
        { snapToGrid: false },
      );

      console.log("Setting flow position:", flowPosition);
      awareness.setLocalStateField("flowPosition", flowPosition);
      lastUpdateRef.current = now;
    },
    [awareness, screenToFlowPosition],
  );

  useEffect(() => {
    if (onCursorUpdate) {
      onCursorUpdate(updateCursor);
    }
  }, [onCursorUpdate, updateCursor]);

  console.log("users", users);
  console.log("awareness", awareness);
  console.log(
    "user states:",
    Array.from(users.entries() as IterableIterator<[string, any]>).map(
      ([clientId, user]) => ({ clientId, user }),
    ),
  );
  console.log("awareness states:", awareness.states);
  return (
    <ViewportPortal>
      {Array.from(users.entries() as IterableIterator<[string, any]>).map(
        ([key, value]) => {
          if (key === awareness.clientID) return null;
          if (!value.flowPosition) return null;

          return (
            <div
              key={key}
              style={{
                position: "absolute",
                left: value.flowPosition.x,
                top: value.flowPosition.y,
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
