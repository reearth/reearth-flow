import { useEffect, useRef, useState } from "react";

import type { Awareness } from "y-protocols/awareness";
import { WebrtcProvider } from "y-webrtc";
import * as Y from "yjs";

import { config } from "@flow/config";
import { CURSOR_COLORS, DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";

import { useUser } from "../gql";

import { yWorkflowConstructor } from "./conversions";
import type { YWorkflow } from "./types";

export default ({
  workflowId,
  projectId,
}: {
  workflowId?: string;
  projectId?: string;
}) => {
  const [undoManager, setUndoManager] = useState<Y.UndoManager | null>(null);
  const [yDocState, setYDocState] = useState<Y.Doc | null>(null);
  const [isSynced, setIsSynced] = useState(false);
  const [yAwareness, setYAwareness] = useState<Awareness | null>(null);
  const { useGetMe } = useUser();
  const { me } = useGetMe();

  useEffect(() => {
    const yDoc = new Y.Doc();
    console.log("📄 yDoc created, GUID:", yDoc.guid);
    
    const cfg = config();
    const { websocket } = cfg;
    let yWebRTCProvider: WebrtcProvider | null = null;

    if (workflowId && projectId) {
      const roomName = `${projectId}:${workflowId}`;
      console.log("🏠 Room name:", roomName);

      // Use public signaling servers (proven to work)
      const signalingUrls = [
        "wss://signaling.yjs.dev",
        "wss://y-webrtc-signaling-eu.herokuapp.com",
      ];
      
      // Optionally try local server too
      if (websocket) {
        signalingUrls.push(websocket.replace(/\/ws\/?$/, "") + "/signaling");
      }

      console.log("📡 Signaling URLs:", signalingUrls);

      // Create WebRTC provider (like yrs-warp example)
      yWebRTCProvider = new WebrtcProvider(roomName, yDoc, { 
        signaling: signalingUrls 
      });

      console.log("✅ WebRTC Provider created:", yWebRTCProvider);

      if (yWebRTCProvider.awareness) {
        const color =
          CURSOR_COLORS[Math.floor(Math.random() * CURSOR_COLORS.length)];
        yWebRTCProvider.awareness.setLocalStateField("color", color);
        yWebRTCProvider.awareness.setLocalStateField(
          "clientId",
          yWebRTCProvider.awareness.clientID,
        );
        yWebRTCProvider.awareness.setLocalStateField(
          "userName",
          me?.name || "Unknown user",
        );

        setYAwareness(yWebRTCProvider.awareness);

        // Monitor awareness changes
        yWebRTCProvider.awareness.on("change", () => {
          const states = yWebRTCProvider?.awareness?.getStates();
          console.log("👤 Awareness updated, clients:", states?.size);
        });
      }

      // Initialize workflow
      const metadata = yDoc.getMap("metadata");
      if (!metadata.get("initialized")) {
        yDoc.transact(() => {
          const yWorkflows = yDoc.getMap<YWorkflow>("workflows");
          if (!yWorkflows.get(DEFAULT_ENTRY_GRAPH_ID)) {
            const yWorkflow = yWorkflowConstructor(
              DEFAULT_ENTRY_GRAPH_ID,
              "Main Workflow",
            );
            yWorkflows.set(DEFAULT_ENTRY_GRAPH_ID, yWorkflow);
            metadata.set("initialized", true);
          }
        });
      }

      // Mark as synced immediately
      setIsSynced(true);

      // Monitor WebRTC peers
      const checkPeers = setInterval(() => {
        const peers = yWebRTCProvider?.room?.webrtcConns?.size || 0;
        console.log("👥 WebRTC peers:", peers);
        if (peers > 0) {
          console.log("✅ P2P established!");
        }
      }, 5000);

      // Cleanup on unmount
      return () => {
        clearInterval(checkPeers);
        setIsSynced(false);
        if (yWebRTCProvider?.awareness) {
          yWebRTCProvider.awareness.setLocalState(null);
        }
        yWebRTCProvider?.destroy();
        setYAwareness(null);
      };
    }

    setYDocState(yDoc);
  }, [projectId, workflowId, me?.name]);

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
        trackedOrigins: new Set([currentUserClientId]),
        captureTimeout: 200,
      });
      setUndoManager(manager);

      return () => {
        manager.destroy();
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

  recursivelyTrackSharedType(yWorkflows);

  return {
    yWorkflows,
    isSynced,
    undoManager,
    undoTrackerActionWrapper,
    yDocState,
    yAwareness,
  };
};

