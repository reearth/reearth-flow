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
  const users = useUsers(awareness, (state) => state);
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
      if (now - lastUpdateRef.current > 66) {
        console.log("Setting point:", [clientX, clientY]);
        awareness.setLocalStateField("point", [clientX, clientY]);
        lastUpdateRef.current = now;
      }
    },
    [awareness],
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
    Array.from(users.entries()).map(([clientId, user]) => ({ clientId, user })),
  );
  console.log("awareness states:", awareness.states);
  return (
    <div
      className="pointer-events-none absolute inset-0"
      style={{
        zIndex: 1000,
      }}>
      {Array.from(users.entries()).map(([key, value]) => {
        if (key === awareness.clientID) return null;
        return <Cursor key={key} color={value.color} point={value.point} />;
      })}
    </div>
  );
};

export default MultiCursor;
