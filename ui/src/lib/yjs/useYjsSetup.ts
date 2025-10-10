import { useEffect, useRef, useState } from "react";

import type { Awareness } from "y-protocols/awareness";
import { WebrtcProvider } from "y-webrtc";
import { WebsocketProvider } from "y-websocket";
import * as Y from "yjs";

import { config } from "@flow/config";
import { CURSOR_COLORS, DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";

import { useAuth } from "../auth";
import { useUser } from "../gql";

import { yWorkflowConstructor } from "./conversions";
import type { YWorkflow } from "./types";

// Sync awareness states between WebSocket (backup) and WebRTC (primary)
function syncAwarenessBetweenProviders(
  websocketAwareness: Awareness,
  webrtcAwareness: Awareness,
) {
  // Sync from WebRTC to WebSocket (for backup)
  webrtcAwareness.on("change", () => {
    const states = webrtcAwareness.getStates();
    states.forEach((state, clientId) => {
      if (clientId !== webrtcAwareness.clientID) {
        websocketAwareness.setLocalStateField(
          `peer_${clientId}`,
          state,
        );
      }
    });
  });

  // Sync from WebSocket to WebRTC (when other users join via WebSocket only)
  websocketAwareness.on("change", () => {
    const states = websocketAwareness.getStates();
    states.forEach((state, clientId) => {
      if (
        clientId !== websocketAwareness.clientID &&
        !webrtcAwareness.getStates().has(clientId)
      ) {
        // Propagate WebSocket-only users to WebRTC awareness
        webrtcAwareness.setLocalStateField(`ws_peer_${clientId}`, state);
      }
    });
  });
}

export default ({
  workflowId,
  projectId,
  isProtected,
  enableWebRTC,
}: {
  workflowId?: string;
  projectId?: string;
  isProtected?: boolean;
  enableWebRTC?: boolean;
}) => {
  const { getAccessToken } = useAuth();

  const [undoManager, setUndoManager] = useState<Y.UndoManager | null>(null);

  const [yDocState, setYDocState] = useState<Y.Doc | null>(null);
  const [isSynced, setIsSynced] = useState(false);
  const [yAwareness, setYAwareness] = useState<Awareness | null>(null);
  const { useGetMe } = useUser();
  const { me } = useGetMe();

  useEffect(() => {
    // Create two separate yDocs: one for WebRTC (primary), one for WebSocket (backup)
    const yDocWebRTC = new Y.Doc(); // Primary doc for WebRTC P2P
    const yDocWebSocket = new Y.Doc(); // Backup doc for WebSocket persistence
    
    console.log("📄 WebRTC yDoc GUID:", yDocWebRTC.guid);
    console.log("📄 WebSocket yDoc GUID:", yDocWebSocket.guid);
    
    const cfg = config();
    const { websocket, enableWebRTC: configWebRTC } = cfg;
    // HARDCODED: Force enable WebRTC for testing
    const shouldEnableWebRTC = true;
    let yWebSocketProvider: WebsocketProvider | null = null;
    let yWebRTCProvider: WebrtcProvider | null = null;
    
    // Sync updates between the two yDocs (bidirectional)
    const syncDocs = () => {
      // WebRTC → WebSocket (for backup)
      yDocWebRTC.on("update", (update: Uint8Array) => {
        Y.applyUpdate(yDocWebSocket, update);
        console.log("📤 Synced WebRTC → WebSocket:", update.length, "bytes");
      });
      
      // WebSocket → WebRTC (for initial load and recovery)
      yDocWebSocket.on("update", (update: Uint8Array, origin: any) => {
        // Only sync if origin is from WebSocket (not from our own sync above)
        if (origin !== "webrtc-sync") {
          Y.applyUpdate(yDocWebRTC, update, "webrtc-sync");
          console.log("📥 Synced WebSocket → WebRTC:", update.length, "bytes");
        }
      });
    };
    
    syncDocs();

    if (workflowId && projectId) {
      (async () => {
        const roomName = `${projectId}:${workflowId}`;

        // Helper to initialize WebRTC
        const initializeWebRTC = () => {
          if (!shouldEnableWebRTC || yWebRTCProvider) return;

          // Use public signaling servers for testing (proven to work)
          const signalingUrls = [
            "wss://signaling.yjs.dev",
            "wss://y-webrtc-signaling-eu.herokuapp.com",
          ];
          
          // Optionally try local server too
          if (websocket) {
            signalingUrls.push(websocket.replace(/\/ws\/?$/, "") + "/signaling");
          }

          // Use WebRTC yDoc (primary)
          yWebRTCProvider = new WebrtcProvider(roomName, yDocWebRTC, { 
            signaling: signalingUrls 
          });

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
            
            // Monitor awareness changes
            yWebRTCProvider.awareness.on("change", () => {
              const states = yWebRTCProvider?.awareness?.getStates();
              console.log("👤 WebRTC Awareness updated, total clients:", states?.size);
              console.log("👥 Clients:", Array.from(states?.keys() || []));
            });
          }
          
          return yWebRTCProvider;
        };

        // Helper to initialize workflow (in WebRTC doc)
        const initializeWorkflow = () => {
          const metadata = yDocWebRTC.getMap("metadata");
          if (!metadata.get("initialized")) {
            yDocWebRTC.transact(() => {
              const yWorkflows = yDocWebRTC.getMap<YWorkflow>("workflows");
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
        };

        // Initialize WebRTC immediately (don't wait for WebSocket)
        console.log("🔧 shouldEnableWebRTC:", shouldEnableWebRTC);
        console.log("🔧 websocket:", websocket);
        console.log("🔧 yWebRTCProvider before init:", yWebRTCProvider);
        
        if (shouldEnableWebRTC) {
          const webrtc = initializeWebRTC();
          console.log("✅ WebRTC Provider initialized:", webrtc);
          console.log("📡 Signaling URLs:", webrtc?.signalingUrls);
          console.log("🏠 Room name:", roomName);
          
          // Monitor WebRTC connection events
          if (webrtc) {
            webrtc.on("synced", (synced: boolean) => {
              console.log("🔄 WebRTC synced:", synced);
            });
            
            webrtc.on("status", (event: { status: string }) => {
              console.log("📊 WebRTC status:", event.status);
            });
            
            // Monitor peers
            const checkPeers = setInterval(() => {
              const peers = webrtc.room?.webrtcConns?.size || 0;
              console.log("👥 WebRTC peers connected:", peers);
              if (peers > 0) {
                console.log("✅ P2P connection established!");
                
                // Check if peers are actually connected
                webrtc.room?.webrtcConns?.forEach((conn: any, peerId: string) => {
                  console.log(`  Peer ${peerId}:`, {
                    connected: conn.connected,
                    destroyed: conn.destroyed,
                  });
                });
              }
            }, 5000);
            
            // Log when data is sent/received via WebRTC
            if (webrtc.room) {
              const originalBroadcast = webrtc.room.broadcast?.bind(webrtc.room);
              if (originalBroadcast) {
                webrtc.room.broadcast = (buf: Uint8Array) => {
                  console.log("📤 WebRTC broadcasting data:", buf.length, "bytes");
                  return originalBroadcast(buf);
                };
              }
            }
            
            // Cleanup interval on destroy
            const originalDestroy = webrtc.destroy.bind(webrtc);
            webrtc.destroy = () => {
              clearInterval(checkPeers);
              originalDestroy();
            };
          }
        }

        // Initialize WebSocket Provider for backup and persistence
        if (websocket) {
          const params: Record<string, string> = {};
          if (isProtected) {
            const token = await getAccessToken();
            params.token = token;
          }

          // Use WebSocket yDoc (backup)
          yWebSocketProvider = new WebsocketProvider(websocket, roomName, yDocWebSocket, {
            params,
            connect: true,
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

          // Use WebRTC awareness if available, otherwise WebSocket
          setYAwareness(yWebRTCProvider?.awareness || yWebSocketProvider.awareness);

          // Set a timeout for WebSocket sync
          const syncTimeout = setTimeout(() => {
            if (!yWebSocketProvider?.synced) {
              console.warn("WebSocket sync timeout, falling back to WebRTC only");
              
              // Ensure WebRTC awareness is set
              if (yWebRTCProvider?.awareness) {
                setYAwareness(yWebRTCProvider.awareness);
              }
              
              // Initialize workflow if needed
              initializeWorkflow();
              
              // Mark as synced to unblock UI
              setIsSynced(true);
            }
          }, 3000); // 3 second timeout

          yWebSocketProvider.once("sync", () => {
            clearTimeout(syncTimeout);
            
            initializeWorkflow();
            setIsSynced(true); // Mark as synced

            // Copy awareness from WebSocket to WebRTC if both exist
            if (yWebRTCProvider?.awareness && yWebSocketProvider.awareness) {
              const wsStates = yWebSocketProvider.awareness.getStates();
              wsStates.forEach((state, clientId) => {
                if (clientId !== yWebSocketProvider.awareness.clientID) {
                  const stateObj = state as Record<string, unknown>;
                  Object.entries(stateObj).forEach(([key, value]) => {
                    yWebRTCProvider?.awareness?.setLocalStateField(
                      `ws_${clientId}_${key}`,
                      value,
                    );
                  });
                }
              });
              
              // Keep using WebRTC awareness (already set)
              setYAwareness(yWebRTCProvider.awareness);
              
              // Setup bidirectional sync
              syncAwarenessBetweenProviders(
                yWebSocketProvider.awareness,
                yWebRTCProvider.awareness,
              );
            }
          });
        } else {
          // No WebSocket - pure WebRTC mode
          if (yWebRTCProvider?.awareness) {
            setYAwareness(yWebRTCProvider.awareness);
          }
          initializeWorkflow();
          setIsSynced(true);
        }
      })();
    }

    // Use WebRTC yDoc as the primary doc
    setYDocState(yDocWebRTC);
    
    // Monitor yDoc updates for debugging
    yDocWebRTC.on("update", (update: Uint8Array, origin: any) => {
      console.log("📝 WebRTC yDoc updated, size:", update.length, "bytes, origin:", origin);
    });

    return () => {
      setIsSynced(false);
      
      // Clear WebSocket awareness (independent)
      if (yWebSocketProvider?.awareness) {
        yWebSocketProvider.awareness.setLocalState(null);
      }
      yWebSocketProvider?.destroy();
      
      // Clear WebRTC awareness (independent)
      if (yWebRTCProvider?.awareness) {
        yWebRTCProvider.awareness.setLocalState(null);
      }
      yWebRTCProvider?.destroy();
      
      setYAwareness(null);
    };
  }, [projectId, workflowId, isProtected, enableWebRTC, me?.name, getAccessToken]);

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

  return {
    yWorkflows,
    isSynced,
    undoManager,
    undoTrackerActionWrapper,
    yDocState,
    yAwareness,
  };
};
