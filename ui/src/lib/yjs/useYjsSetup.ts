import { useEffect, useRef, useState } from "react";
import { WebsocketProvider } from "y-websocket";
import * as Y from "yjs";

import { config } from "@flow/config";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";

import { useAuth } from "../auth";

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

  const [undoManager, setUndoManager] = useState<Y.UndoManager | null>(null);

  const [yDocState, setYDocState] = useState<Y.Doc | null>(null);
  const [isSynced, setIsSynced] = useState(false);
  
  const yWebSocketProviderRef = useRef<WebsocketProvider | null>(null);

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

        yWebSocketProvider = new WebsocketProvider(
          websocket,
          `${projectId}:${workflowId}`,
          yDoc,
          {
            params,
          },
        );
        
        yWebSocketProviderRef.current = yWebSocketProvider;

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
      yWebSocketProvider?.destroy();
      yWebSocketProviderRef.current = null;
    };
  }, [projectId, workflowId, isProtected, getAccessToken]);

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

  useEffect(() => {
    if (yWorkflows) {
      const manager = new Y.UndoManager(yWorkflows, {
        trackedOrigins: new Set([currentUserClientId]), // Only track local changes
        captureTimeout: 200, // default is 500. 200ms is a good balance between performance and user experience
      });
      setUndoManager(manager);

      return () => {
        manager.destroy(); // Clean up UndoManager on component unmount
        setUndoManager(null);
      };
    }
  }, [yWorkflows, currentUserClientId]);

  const observedMapsRef = useRef(new WeakSet());

  function recursivelyTrackSharedType(sharedType?: Y.Map<any>): void {
    if (!sharedType) return;
    if (observedMapsRef.current.has(sharedType)) return;
    observedMapsRef.current.add(sharedType);

    undoManager?.addToScope([sharedType]);

    if (sharedType instanceof Y.Map) {
      sharedType.forEach((value: any) => {
        if (value instanceof Y.Map) {
          recursivelyTrackSharedType(value);
        }
      });

      sharedType.observe((event: Y.YMapEvent<any>) => {
        event.changes.keys.forEach((change: any, key: string) => {
          if (change.action === "add" || change.action === "update") {
            const newValue: any = sharedType.get(key);
            if (newValue instanceof Y.Map) {
              recursivelyTrackSharedType(newValue);
            }
          }
        });
      });
    }
  }

  // Start the recursive tracking
  recursivelyTrackSharedType(yWorkflows);

  const awareness = yWebSocketProviderRef.current?.awareness;

  return {
    yWorkflows,
    isSynced,
    undoManager,
    undoTrackerActionWrapper,
    yDocState,
    awareness,
  };
};
