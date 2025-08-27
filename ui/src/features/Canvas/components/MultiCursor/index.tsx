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
};

type MultiCursorProps = {
  yDoc: Y.Doc | null;
  awareness: any;
  currentUserName?: string;
  onCursorUpdate?: (updateFn: (clientX: number, clientY: number) => void) => void;
};

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

const MultiCursor: React.FC<MultiCursorProps> = ({
  yDoc,
  awareness,
  currentUserName,
  onCursorUpdate,
}) => {
  const [cursors, setCursors] = useState<Map<number, Cursor>>(new Map());
  const { screenToFlowPosition, flowToScreenPosition } = useReactFlow();

  // Handle awareness updates
  useEffect(() => {
    if (!yDoc || !awareness) return;

    // Expose awareness to window for debugging
    if (typeof window !== "undefined") {
      (window as any).awareness = awareness;
      (window as any).yDoc = yDoc;
    }

    const handleAwarenessUpdate = () => {
      const states = awareness.getStates();
      const newCursors = new Map<number, Cursor>();

      console.log("Awareness update - Total clients:", states.size);
      console.log("My client ID:", yDoc.clientID);

      states.forEach((state: any, clientId: number) => {
        console.log(`Client ${clientId} state:`, state);

        // Skip self
        if (clientId === yDoc.clientID) return;

        if (state.cursor) {
          // Convert flow coordinates to screen coordinates
          const screenPos = flowToScreenPosition({
            x: state.cursor.x,
            y: state.cursor.y,
          });
          newCursors.set(clientId, {
            id: clientId,
            x: screenPos.x,
            y: screenPos.y,
            name: state.user?.name || `User ${clientId}`,
            color: getUserColor(clientId),
          });
        }
      });
      console.log("Rendering cursors:", newCursors);

      console.log("Active cursors:", newCursors.size);
      setCursors(newCursors);
    };

    awareness.on("update", handleAwarenessUpdate);

    // Initial update
    handleAwarenessUpdate();

    return () => {
      awareness.off("update", handleAwarenessUpdate);
    };
  }, [yDoc, awareness, flowToScreenPosition]);

  // Shared function to update cursor position
  const updateCursorPosition = useCallback(
    (clientX: number, clientY: number) => {
      if (!yDoc || !awareness) return;

      // Find the ReactFlow element
      const reactFlowElement = document.querySelector(".react-flow");
      if (!reactFlowElement) return;

      const rect = reactFlowElement.getBoundingClientRect();

      // Check if mouse is within ReactFlow bounds
      if (
        clientX >= rect.left &&
        clientX <= rect.right &&
        clientY >= rect.top &&
        clientY <= rect.bottom
      ) {
        const x = clientX - rect.left;
        const y = clientY - rect.top;

        // Convert screen coordinates to flow coordinates
        const flowPos = screenToFlowPosition({ x, y });

        console.log("Sending cursor position:", flowPos);
        console.log("User name:", currentUserName || `User ${yDoc.clientID}`);

        // Set local state with both cursor and user info
        awareness.setLocalState({
          cursor: flowPos,
          user: {
            name: currentUserName || `User ${yDoc.clientID}`,
          },
        });
      }
    },
    [yDoc, awareness, currentUserName, screenToFlowPosition],
  );

  // Expose cursor update function to parent component
  useEffect(() => {
    if (onCursorUpdate) {
      const cursorUpdateFn = (clientX: number, clientY: number) => {
        updateCursorPosition(clientX, clientY);
      };
      onCursorUpdate(cursorUpdateFn);
    }
  }, [onCursorUpdate, updateCursorPosition]);

  // Use multiple event types to capture mouse position during all interactions
  useEffect(() => {
    if (!yDoc || !awareness) return;

    let lastMouseX = 0;
    let lastMouseY = 0;
    let animationFrameId: number;
    let isMouseOver = false;

    const updateLastPosition = (event: MouseEvent | PointerEvent) => {
      lastMouseX = event.clientX;
      lastMouseY = event.clientY;
      isMouseOver = true;
    };

    const handleMouseLeave = () => {
      isMouseOver = false;
      // Clear cursor when mouse leaves entirely
      const currentState = awareness.getLocalState();
      awareness.setLocalState({
        ...currentState,
        cursor: null,
      });
    };

    const pollMousePosition = () => {
      if (isMouseOver) {
        updateCursorPosition(lastMouseX, lastMouseY);
      }
      animationFrameId = requestAnimationFrame(pollMousePosition);
    };

    // Start polling
    animationFrameId = requestAnimationFrame(pollMousePosition);

    // Listen for multiple event types to catch all mouse movements
    document.addEventListener('mousemove', updateLastPosition, true);
    document.addEventListener('mousedown', updateLastPosition, true);
    document.addEventListener('mouseup', updateLastPosition, true);
    
    // Pointer events can sometimes capture what mouse events miss
    document.addEventListener('pointermove', updateLastPosition, true);
    document.addEventListener('pointerdown', updateLastPosition, true);
    document.addEventListener('pointerup', updateLastPosition, true);
    
    // Listen for mouseleave to clear cursor
    document.addEventListener('mouseleave', handleMouseLeave);

    return () => {
      cancelAnimationFrame(animationFrameId);
      document.removeEventListener('mousemove', updateLastPosition, true);
      document.removeEventListener('mousedown', updateLastPosition, true);
      document.removeEventListener('mouseup', updateLastPosition, true);
      document.removeEventListener('pointermove', updateLastPosition, true);
      document.removeEventListener('pointerdown', updateLastPosition, true);
      document.removeEventListener('pointerup', updateLastPosition, true);
      document.removeEventListener('mouseleave', handleMouseLeave);
    };
  }, [updateCursorPosition, awareness, yDoc]);

  return (
    <div
      className="pointer-events-none absolute inset-0"
      style={{
        zIndex: 1000,
      }}>
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
