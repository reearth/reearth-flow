import { useCallback, useEffect, useRef, useState } from "react";
import type { Awareness } from "y-protocols/awareness";
import { WebsocketProvider } from "y-websocket";
import * as Y from "yjs";

import { config } from "@flow/config";
import { CURSOR_COLORS, DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";

import { useAuth } from "../auth";
import { useUser } from "../gql";

import { yWorkflowConstructor } from "./conversions";
import type { YWorkflow } from "./types";

export default ({
  workflowId,
  projectId,
  isProtected,
}: {
  workflowId?: string;
  projectId?: string;
  isProtected?: boolean;
}) => {
  const { getAccessToken } = useAuth();

  const undoManagersRef = useRef<Map<string, Y.UndoManager>>(new Map());

  const [yDocState, setYDocState] = useState<Y.Doc | null>(null);
  const [isSynced, setIsSynced] = useState(false);
  const [yAwareness, setYAwareness] = useState<Awareness | null>(null);
  const { useGetMe } = useUser();
  const { me } = useGetMe();

  useEffect(() => {
    const yDoc = new Y.Doc();
    const { websocket } = config();
    let yWebSocketProvider: WebsocketProvider | null = null;

    if (workflowId && websocket && projectId) {
      (async () => {
        const params: Record<string, string> = {};
        if (isProtected) {
          const token = await getAccessToken();
          params.token = token;
        }

        const roomName = `${projectId}:${workflowId}`;

        yWebSocketProvider = new WebsocketProvider(websocket, roomName, yDoc, {
          params,
        });

        if (
          yWebSocketProvider.awareness &&
          !yWebSocketProvider.awareness.getLocalState()?.color
        ) {
          const color =
            CURSOR_COLORS[Math.floor(Math.random() * CURSOR_COLORS.length)];
          yWebSocketProvider.awareness.setLocalStateField("color", color);
          yWebSocketProvider.awareness.setLocalStateField(
            "clientId",
            yWebSocketProvider.awareness.clientID,
          );
          yWebSocketProvider.awareness.setLocalStateField(
            "userName",
            me?.name || "Unknown user",
          );
        }

        setYAwareness(yWebSocketProvider.awareness);

        yWebSocketProvider.once("sync", () => {
          const metadata = yDoc.getMap("metadata");
          if (!metadata.get("initialized")) {
            // Within a transaction, set the flag and perform initialization.
            yDoc.transact(() => {
              const yWorkflows = yDoc.getMap<YWorkflow>("workflows");
              // This check is only necessary to avoid duplicate workflows on older projects.
              if (yWorkflows.get(DEFAULT_ENTRY_GRAPH_ID)) return;
              // Only one client should set this flag.
              if (!metadata.get("initialized")) {
                const yWorkflow = yWorkflowConstructor(
                  DEFAULT_ENTRY_GRAPH_ID,
                  "Main Workflow",
                );
                yWorkflows.set(DEFAULT_ENTRY_GRAPH_ID, yWorkflow);
                metadata.set("initialized", true);
              }
            });
          }
          setIsSynced(true); // Mark as synced
        });
      })();
    }

    setYDocState(yDoc);

    return () => {
      setIsSynced(false);
      // Clear awareness state before destroying
      if (yWebSocketProvider?.awareness) {
        yWebSocketProvider?.awareness.setLocalState(null);
      }
      yWebSocketProvider?.destroy();
      setYAwareness(null);
    };
  }, [projectId, workflowId, isProtected, me, getAccessToken]);

  const currentUserClientId = yDocState?.clientID;

  const yWorkflows = yDocState?.getMap<YWorkflow>("workflows");

  const undoTrackerActionWrapper = (
    callback: () => void,
    originPrepend?: string,
  ) => {
    const origin = originPrepend
      ? `${originPrepend}-${yDocState?.clientID}`
      : yDocState?.clientID;
    yDocState?.transact(callback, origin);
  };

  const recursivelyTrackSharedType = useCallback(
    (
      observedMapsRef: WeakSet<Y.Map<unknown>>,
      manager: Y.UndoManager,
      sharedType?: Y.Map<any>,
    ): void => {
      if (!sharedType) return;
      if (observedMapsRef.has(sharedType)) return;
      observedMapsRef.add(sharedType);

      manager.addToScope([sharedType]);

      if (sharedType instanceof Y.Map) {
        sharedType.forEach((value: any) => {
          if (value instanceof Y.Map) {
            recursivelyTrackSharedType(observedMapsRef, manager, value);
          }
        });

        sharedType.observe((event: Y.YMapEvent<any>) => {
          event.changes.keys.forEach((change: any, key: string) => {
            if (change.action === "add" || change.action === "update") {
              const newValue: any = sharedType.get(key);
              if (newValue instanceof Y.Map) {
                recursivelyTrackSharedType(observedMapsRef, manager, newValue);
              }
            }
          });
        });
      }
    },
    [],
  );

  const handleWorkflowsChange = useCallback(
    (
      currentUserClientId: number,
      yWorkflows: Y.Map<YWorkflow>,
      observedMapsRef: WeakSet<any>,
    ) => {
      const currentManagers = undoManagersRef.current;
      const newManagers = new Map<string, Y.UndoManager>();

      yWorkflows.forEach((workflow, workflowId) => {
        let manager = currentManagers.get(workflowId);

        if (!manager) {
          // Create separate undo manager for each workflow
          manager = new Y.UndoManager([workflow], {
            trackedOrigins: new Set([currentUserClientId]), // Only track local changes
            captureTimeout: 200, // default is 500. 200ms is a good balance between performance and user experience
          });

          // Recursively track all nested shared types
          recursivelyTrackSharedType(observedMapsRef, manager, workflow);
        }

        newManagers.set(workflowId, manager);
      });

      // Cleanup managers for workflows that are deleted
      currentManagers.forEach((manager, workflowId) => {
        if (!newManagers.has(workflowId)) {
          manager.destroy();
        }
      });

      undoManagersRef.current = newManagers;
    },
    [recursivelyTrackSharedType],
  );

  useEffect(() => {
    if (!yWorkflows || !currentUserClientId) return;

    const observedMapsRef = new WeakSet();

    // Initial setup
    handleWorkflowsChange(currentUserClientId, yWorkflows, observedMapsRef);

    // Observe workflows map for additions/removals
    const workflowsObserver = () => {
      handleWorkflowsChange(currentUserClientId, yWorkflows, observedMapsRef);
    };

    yWorkflows.observe(workflowsObserver);

    return () => {
      yWorkflows.unobserve(workflowsObserver);
      // Clean up UndoManagers on component unmount
      undoManagersRef.current.forEach((manager) => manager.destroy());
      undoManagersRef.current = new Map();
    };
  }, [yWorkflows, currentUserClientId, handleWorkflowsChange]);

  const getUndoManager = (workflowId: string): Y.UndoManager | null => {
    return undoManagersRef.current.get(workflowId) ?? null;
  };

  return {
    yWorkflows,
    isSynced,
    getUndoManager,
    undoTrackerActionWrapper,
    yDocState,
    yAwareness,
  };
};
