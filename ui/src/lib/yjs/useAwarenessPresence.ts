import { useReactFlow, useViewport } from "@xyflow/react";
import { throttle } from "lodash-es";
import { MouseEvent, useCallback, useEffect, useMemo, useRef } from "react";
import { useUsers, useSelf } from "y-presence";
import type { Awareness } from "y-protocols/awareness";

import type {
  AwarenessDialog,
  AwarenessEchoMap,
  AwarenessNodePicker,
  AwarenessSelectionsMap,
  AwarenessSubEditor,
  AwarenessUser,
  Node,
} from "@flow/types";

export default function useAwarenessPresence({
  yAwareness,
  openNode,
  selectedNodeIds,
}: {
  yAwareness: Awareness;
  openNode: Node | undefined;
  selectedNodeIds: string[];
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
    // Carried so the collaboration popover can label your own row the same way
    // it labels everyone else's.
    openNodeId: rawSelf?.openNodeId,
    openSubEditor: rawSelf?.openSubEditor,
    openDialog: rawSelf?.openDialog,
    openNodePicker: rawSelf?.openNodePicker,
    openWorkflowVariablesDialog: rawSelf?.openWorkflowVariablesDialog,
    selectedVersionSnapshot: rawSelf?.selectedVersionSnapshot,
    followingClientId: rawSelf?.followingClientId,
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
    (e: MouseEvent) => {
      const flowPosition = screenToFlowPosition(
        { x: e.clientX, y: e.clientY },
        { snapToGrid: false },
      );

      yAwareness.setLocalStateField("cursor", flowPosition);
      yAwareness.setLocalStateField("viewport", latestViewportRef.current);

      if (!e.shiftKey || e.button !== 0) {
        return;
      }

      e.preventDefault();
      e.stopPropagation();

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

  // Stable reference — only produces a new object when selection content changes,
  // so cursor moves on rawUsers don't invalidate the EditorContext value.
  const prevSelectionsKeyRef = useRef("");
  const awarenessSelectionsMapRef = useRef<AwarenessSelectionsMap>({});
  const awarenessSelectionsMap = useMemo(() => {
    let key = "";
    const map: AwarenessSelectionsMap = {};

    rawUsers.forEach((state, clientId) => {
      if (clientId === yAwareness.clientID) return;
      const user = state;
      if (!user.userName || !user.selectedNodeIds?.length) return;

      key += `${clientId}:${user.selectedNodeIds.join(",")}|`;
      user.selectedNodeIds.forEach((nodeId: string) => {
        if (!map[nodeId]) map[nodeId] = [];
        map[nodeId].push({ color: user.color, userName: user.userName });
      });
    });

    if (key === prevSelectionsKeyRef.current) {
      return awarenessSelectionsMapRef.current;
    }

    prevSelectionsKeyRef.current = key;
    awarenessSelectionsMapRef.current = map;
    return map;
  }, [rawUsers, yAwareness.clientID]);

  // Generic "who is on this element" channel. One awareness field backs every
  // surface — dropdowns, list rows, menu items — so adding a new one is a
  // `useEchoPresence(key, isOpen)` call rather than new plumbing.
  const activeElementsRef = useRef<Set<string>>(new Set());
  const setEchoElement = useCallback(
    (key: string, active: boolean) => {
      const set = activeElementsRef.current;
      if (active) {
        if (set.has(key)) return;
        set.add(key);
      } else {
        if (!set.has(key)) return;
        set.delete(key);
      }
      yAwareness.setLocalStateField(
        "activeElements",
        set.size ? Array.from(set) : null,
      );
    },
    [yAwareness],
  );

  // Same stable-reference treatment as awarenessSelectionsMap: cursor moves
  // must not invalidate the EditorContext value.
  const prevEchoKeyRef = useRef("");
  const awarenessEchoMapRef = useRef<AwarenessEchoMap>({});
  const awarenessEchoMap = useMemo(() => {
    let key = "";
    const map: AwarenessEchoMap = {};

    rawUsers.forEach((state, clientId) => {
      if (clientId === yAwareness.clientID) return;
      const user = state as AwarenessUser;
      if (!user.userName || !user.activeElements?.length) return;

      key += `${clientId}:${user.activeElements.join(",")}|`;
      user.activeElements.forEach((elementKey: string) => {
        if (!map[elementKey]) map[elementKey] = [];
        map[elementKey].push({ color: user.color, userName: user.userName });
      });
    });

    if (key === prevEchoKeyRef.current) {
      return awarenessEchoMapRef.current;
    }

    prevEchoKeyRef.current = key;
    awarenessEchoMapRef.current = map;
    return map;
  }, [rawUsers, yAwareness.clientID]);

  const handleParamFieldFocus = useCallback(
    (fieldId: string | null) => {
      yAwareness.setLocalStateField("focusedParamField", fieldId);
    },
    [yAwareness],
  );

  const handleWorkflowVarDialogOpen = useCallback(() => {
    yAwareness.setLocalStateField("openWorkflowVariablesDialog", true);
  }, [yAwareness]);

  const handleWorkflowVarDialogClose = useCallback(() => {
    yAwareness.setLocalStateField("openWorkflowVariablesDialog", null);
    yAwareness.setLocalStateField("focusedVariableId", null);
    yAwareness.setLocalStateField("focusedVariableField", null);
    yAwareness.setLocalStateField("editingVariableId", null);
  }, [yAwareness]);

  const handleWorkflowVarFieldFocus = useCallback(
    (variableId: string | null, field: string | null) => {
      yAwareness.setLocalStateField("focusedVariableId", variableId);
      yAwareness.setLocalStateField("focusedVariableField", field);
    },
    [yAwareness],
  );

  const handleWorkflowVarEditStart = useCallback(
    (variableId: string | null) => {
      yAwareness.setLocalStateField("editingVariableId", variableId);
    },
    [yAwareness],
  );

  const handleUserFocusedElement = useCallback(
    (isOpen: boolean) => {
      yAwareness.setLocalStateField("focusedElement", isOpen);
    },
    [yAwareness],
  );

  const handleDialogAwareness = useCallback(
    (
      dialog: AwarenessDialog | null,
      ownedDialogs: readonly AwarenessDialog[],
    ) => {
      if (dialog === null) {
        // The Homebar and the overlay each keep their own `showDialog`, so a
        // close from one must not clear a dialog the other still has open.
        const current = yAwareness.getLocalState()?.openDialog as
          | AwarenessDialog
          | null
          | undefined;
        if (current && !ownedDialogs.includes(current)) return;
      }
      yAwareness.setLocalStateField("openDialog", dialog);
    },
    [yAwareness],
  );

  const handleSubEditorAwareness = useCallback(
    (subEditor: AwarenessSubEditor | null) => {
      yAwareness.setLocalStateField("openSubEditor", subEditor);
    },
    [yAwareness],
  );

  const handleNodePickerAwareness = useCallback(
    (nodePicker: AwarenessNodePicker | null) => {
      yAwareness.setLocalStateField("openNodePicker", nodePicker);
    },
    [yAwareness],
  );

  const handleVersionSnapshotAwareness = useCallback(
    (snapshotNumber: number | null) => {
      yAwareness.setLocalStateField("selectedVersionSnapshot", snapshotNumber);
    },
    [yAwareness],
  );

  useEffect(() => {
    const handleWindowPointerMove = (event: PointerEvent) => {
      if (yAwareness.getLocalState()?.focusedElement) return;
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

  useEffect(() => {
    yAwareness.setLocalStateField(
      "selectedNodeIds",
      selectedNodeIds.length > 0 ? selectedNodeIds : null,
    );
  }, [selectedNodeIds, yAwareness]);

  useEffect(() => {
    const openNodeId = openNode?.id ?? null;

    const localOpenNodeId = yAwareness.getLocalState()?.openNodeId;

    if (openNodeId === localOpenNodeId) {
      return;
    }

    yAwareness.setLocalStateField("openNodeId", openNodeId);
    yAwareness.setLocalStateField("focusedParamField", null);
    yAwareness.setLocalStateField("openSubEditor", null);
  }, [openNode, yAwareness]);

  return {
    self,
    users,
    awarenessSelectionsMap,
    awarenessEchoMap,
    setEchoElement,
    handlePointerDown,
    handleParamFieldFocus,
    handleWorkflowVarDialogOpen,
    handleWorkflowVarDialogClose,
    handleWorkflowVarFieldFocus,
    handleWorkflowVarEditStart,
    handleUserFocusedElement,
    handleDialogAwareness,
    handleSubEditorAwareness,
    handleNodePickerAwareness,
    handleVersionSnapshotAwareness,
    setDraggingEdge,
    clearDraggingEdge,
  };
}
