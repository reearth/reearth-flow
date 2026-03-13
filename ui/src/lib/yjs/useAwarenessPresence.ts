import { useReactFlow, useViewport } from "@xyflow/react";
import { throttle } from "lodash-es";
import { useCallback, useEffect, useMemo, useRef } from "react";
import { useUsers, useSelf } from "y-presence";
import type { Awareness } from "y-protocols/awareness";

import type { AwarenessUser } from "@flow/types";

type PointerDownEvent = React.PointerEvent<Element>;

export default function useAwarenessPresence({
  yAwareness,
}: {
  yAwareness: Awareness;
}) {
  const rawSelf = useSelf(yAwareness);
  const rawUsers = useUsers(yAwareness);
  const { screenToFlowPosition } = useReactFlow();
  const { x, y, zoom } = useViewport();

  const isSelectingRef = useRef(false);
  const selectionStartRef = useRef<{ x: number; y: number } | null>(null);

  const latestViewportRef = useRef({ x, y, zoom });
  latestViewportRef.current = { x, y, zoom };

  const self: AwarenessUser = {
    clientId: rawSelf?.clientID,
    userName: rawSelf?.userName || "Unknown user",
    color: rawSelf?.color || "#ffffff",
    cursor: rawSelf?.cursor || { x: 0, y: 0 },
    viewport: rawSelf?.viewport,
    currentWorkflowId: rawSelf?.currentWorkflowId,
    selectionRect: rawSelf?.selectionRect ?? undefined,
  };

  const users = Array.from(
    rawUsers.entries() as IterableIterator<[number, AwarenessUser]>,
  )
    .filter(([key]) => key !== yAwareness.clientID)
    .reduce<Record<string, AwarenessUser>>((acc, [key, value]) => {
      if (!value.userName) return acc;
      acc[key.toString()] = value;
      return acc;
    }, {});

  const throttledPresenceUpdate = useMemo(
    () =>
      throttle(
        (clientX: number, clientY: number) => {
          const flowPosition = screenToFlowPosition(
            { x: clientX, y: clientY },
            { snapToGrid: false },
          );

          yAwareness.setLocalStateField("cursor", flowPosition);
          yAwareness.setLocalStateField("viewport", latestViewportRef.current);
        },
        32,
        { leading: true, trailing: true },
      ),
    [screenToFlowPosition, yAwareness],
  );

  const updateSelectionRect = useCallback(
    (clientX: number, clientY: number) => {
      if (!isSelectingRef.current || !selectionStartRef.current) return;

      const flowPosition = screenToFlowPosition(
        { x: clientX, y: clientY },
        { snapToGrid: false },
      );

      const nextRect = {
        startX: selectionStartRef.current.x,
        startY: selectionStartRef.current.y,
        currentX: flowPosition.x,
        currentY: flowPosition.y,
      };

      yAwareness.setLocalStateField("selectionRect", nextRect);
    },
    [screenToFlowPosition, yAwareness],
  );

  const handlePointerDown = useCallback(
    (event: PointerDownEvent) => {
      const flowPosition = screenToFlowPosition(
        { x: event.clientX, y: event.clientY },
        { snapToGrid: false },
      );

      yAwareness.setLocalStateField("cursor", flowPosition);
      yAwareness.setLocalStateField("viewport", latestViewportRef.current);

      if (!event.shiftKey || event.button !== 0) {
        return;
      }

      event.preventDefault();
      event.stopPropagation();

      isSelectingRef.current = true;
      selectionStartRef.current = {
        x: flowPosition.x,
        y: flowPosition.y,
      };

      yAwareness.setLocalStateField("selectionRect", {
        startX: flowPosition.x,
        startY: flowPosition.y,
        currentX: flowPosition.x,
        currentY: flowPosition.y,
      });
    },
    [screenToFlowPosition, yAwareness],
  );

  const clearSelectionRect = useCallback(() => {
    isSelectingRef.current = false;
    selectionStartRef.current = null;
    yAwareness.setLocalStateField("selectionRect", null);
  }, [yAwareness]);

  const setDraggingEdge = useCallback(
    (
      nodeId: string,
      handleId: string | null,
      handleType: "source" | "target" | null,
    ) => {
      yAwareness.setLocalStateField("draggingEdge", {
        nodeId,
        handleId,
        handleType,
      });
    },
    [yAwareness],
  );

  const clearDraggingEdge = useCallback(() => {
    yAwareness.setLocalStateField("draggingEdge", null);
  }, [yAwareness]);

  const setSelectedNodes = useCallback(
    (nodeIds: string[]) => {
      yAwareness.setLocalStateField(
        "selectedNodeIds",
        nodeIds.length > 0 ? nodeIds : null,
      );
    },
    [yAwareness],
  );

  useEffect(() => {
    const handleWindowPointerMove = (event: PointerEvent) => {
      throttledPresenceUpdate(event.clientX, event.clientY);
      const awarenessStates = yAwareness.getStates();
      if (awarenessStates.size > 1) {
        throttledPresenceUpdate(event.clientX, event.clientY);
      }

      if (isSelectingRef.current) {
        updateSelectionRect(event.clientX, event.clientY);
      }
    };

    const handleWindowPointerUp = () => {
      clearSelectionRect();
    };

    const handleWindowBlur = () => {
      clearSelectionRect();
    };

    window.addEventListener("pointermove", handleWindowPointerMove);
    window.addEventListener("pointerup", handleWindowPointerUp);
    window.addEventListener("blur", handleWindowBlur);

    return () => {
      window.removeEventListener("pointermove", handleWindowPointerMove);
      window.removeEventListener("pointerup", handleWindowPointerUp);
      window.removeEventListener("blur", handleWindowBlur);
      throttledPresenceUpdate.cancel();
    };
  }, [
    throttledPresenceUpdate,
    updateSelectionRect,
    clearSelectionRect,
    yAwareness,
  ]);

  return {
    self,
    users,
    handlePointerDown,
    isSelectingRef,
    selectionStartRef,
    setDraggingEdge,
    clearDraggingEdge,
    setSelectedNodes,
  };
}
