import { useReactFlow } from "@xyflow/react";
import { useCallback, useEffect, useState } from "react";
import * as Y from "yjs";

import CursorComponent from "./CursorComponent";

type Cursor = {
  id: number;
  x: number;
  y: number;
  name: string;
  color: string;
}

type MultiCursorProps = {
  yDoc: Y.Doc | null;
  awareness: any;
  currentUserName?: string;
}

// Function to generate consistent color from user ID
const getUserColor = (userId: number): string => {
  const colors = [
    "#ef4444", // red
    "#f59e0b", // amber
    "#10b981", // emerald
    "#3b82f6", // blue
    "#8b5cf6", // violet
    "#ec4899", // pink
    "#06b6d4", // cyan
    "#84cc16", // lime
  ];
  return colors[userId % colors.length];
};

const MultiCursor: React.FC<MultiCursorProps> = ({ yDoc, awareness, currentUserName }) => {
  const [cursors, setCursors] = useState<Map<number, Cursor>>(new Map());
  const { screenToFlowPosition } = useReactFlow();

  // Handle awareness updates
  useEffect(() => {
    if (!yDoc || !awareness) return;

    const handleAwarenessUpdate = () => {
      const states = awareness.getStates();
      const newCursors = new Map<number, Cursor>();

      states.forEach((state: any, clientId: number) => {
        // Skip self
        if (clientId === yDoc.clientID) return;

        if (state.cursor) {
          newCursors.set(clientId, {
            id: clientId,
            x: state.cursor.x,
            y: state.cursor.y,
            name: state.user?.name || `User ${clientId}`,
            color: getUserColor(clientId),
          });
        }
      });

      setCursors(newCursors);
    };

    awareness.on("update", handleAwarenessUpdate);
    
    // Initial update
    handleAwarenessUpdate();

    return () => {
      awareness.off("update", handleAwarenessUpdate);
    };
  }, [yDoc, awareness]);

  // Broadcast own cursor position
  const handleMouseMove = useCallback(
    (event: React.MouseEvent) => {
          if (!yDoc || !awareness) return;

      const rect = event.currentTarget.getBoundingClientRect();
      const x = event.clientX - rect.left;
      const y = event.clientY - rect.top;
      
      // Convert screen coordinates to flow coordinates
      const flowPos = screenToFlowPosition({ x, y });

      awareness.setLocalStateField("cursor", flowPos);
      awareness.setLocalStateField("user", {
        name: currentUserName || `User ${yDoc.clientID}`,
      });
    },
    [yDoc, awareness, currentUserName, screenToFlowPosition]
  );

  // Clear cursor when mouse leaves
  const handleMouseLeave = useCallback(() => {
    if (!yDoc || !awareness) return;

    awareness.setLocalStateField("cursor", null);
  }, [awareness, yDoc]);

  return (
    <div
      className="absolute inset-0"
      onMouseMove={handleMouseMove}
      onMouseLeave={handleMouseLeave}>
      {Array.from(cursors.values()).map((cursor) => (
        <CursorComponent
          key={cursor.id}
          x={cursor.x}
          y={cursor.y}
          color={cursor.color}
          name={cursor.name}
        />
      ))}
    </div>
  );
};

export default MultiCursor;