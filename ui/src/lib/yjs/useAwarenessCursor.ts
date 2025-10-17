import { useReactFlow, useViewport } from "@xyflow/react";
import { throttle } from "lodash-es";
import { MouseEvent, useCallback, useMemo, useRef } from "react";
import { useUsers, useSelf } from "y-presence";
import type { Awareness } from "y-protocols/awareness";

import { AwarenessUser } from "@flow/types";

export default ({ yAwareness }: { yAwareness: Awareness }) => {
  const rawSelf = useSelf(yAwareness);
  const rawUsers = useUsers(yAwareness);
  const { screenToFlowPosition } = useReactFlow();

  const self: AwarenessUser = {
    clientId: rawSelf?.clientID,
    userName: rawSelf?.userName || "Unknown user",
    color: rawSelf?.color || "#ffffff",
    cursor: rawSelf?.cursor || { x: 0, y: 0 },
  };

  const users = Array.from(
    rawUsers.entries() as IterableIterator<[number, AwarenessUser]>,
  )
    .filter(([key]) => key !== yAwareness?.clientID)
    .reduce<Record<string, AwarenessUser>>((acc, [key, value]) => {
      if (!value.userName) {
        return acc;
      }
      acc[key.toString()] = value;
      return acc;
    }, {});

  const { x, y, zoom } = useViewport();

  const latestViewportRef = useRef<{ x: number; y: number; zoom: number }>({
    x,
    y,
    zoom,
  });

  latestViewportRef.current = { x, y, zoom };

  const throttledMouseMove = useMemo(
    () =>
      throttle(
        (
          event: MouseEvent<Element, globalThis.MouseEvent>,
          awareness: Awareness,
          positionFn: typeof screenToFlowPosition,
        ) => {
          const flowPosition = positionFn(
            {
              x: event.clientX,
              y: event.clientY,
            },
            { snapToGrid: false },
          );

          awareness.setLocalStateField("cursor", flowPosition);
          awareness.setLocalStateField("viewport", latestViewportRef.current);
        },
        32,
        { leading: true, trailing: true },
      ),
    [],
  );

  const handlePaneMouseMove = useCallback(
    (event: MouseEvent) => {
      if (Object.keys(users).length === 0) return;
      throttledMouseMove(event, yAwareness, screenToFlowPosition);
    },
    [yAwareness, users, screenToFlowPosition, throttledMouseMove],
  );

  return { self, users, handlePaneMouseMove };
};
