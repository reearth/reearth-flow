import * as Y from "yjs";

import { sleep } from "../utils";

import type { FlowMessage } from "./types";

export type AccessTokenProvider = () => Promise<string> | string;

enum MessageType {
  UPDATE = 1,
  SYNC = 2,
}

function createBinaryMessage(type: MessageType, data: Uint8Array): Uint8Array {
  const message = new Uint8Array(data.length + 1);
  message[0] = type;
  message.set(data, 1);
  return message;
}

export class SocketYjsManager {
  protected ws!: WebSocket;
  protected doc: Y.Doc;
  protected socketReady = false;
  protected firstSyncComplete = false;
  protected accessTokenProvider: AccessTokenProvider | undefined;
  protected projectId: string | undefined;
  protected onUpdateHandlers: ((update: Uint8Array) => void)[] = [];
  protected reconnectAttempts = 0;
  protected maxReconnectAttempts = 5;
  protected reconnectDelay = 1000;
  protected reconnectTimer: ReturnType<typeof setTimeout> | null = null;

  constructor(doc: Y.Doc) {
    this.doc = doc;

    // Bind methods
    this.onConnectionEstablished = this.onConnectionEstablished.bind(this);
    this.onConnectionDisconnect = this.onConnectionDisconnect.bind(this);
    this.onConnectionError = this.onConnectionError.bind(this);
    this.onAuthenticateRequest = this.onAuthenticateRequest.bind(this);
    this.onDocUpdate = this.onDocUpdate.bind(this);
    this.onReady = this.onReady.bind(this);
    this.onPeerUpdate = this.onPeerUpdate.bind(this);
    this.handleMessage = this.handleMessage.bind(this);
    this.reconnect = this.reconnect.bind(this);
  }

  public getDoc(): Y.Doc {
    return this.doc;
  }

  // Replace the setupSocket method
  async setupSocket(data: {
    url: string;
    roomId: string;
    projectId: string;
    accessTokenProvider: AccessTokenProvider;
  }) {
    this.accessTokenProvider = data.accessTokenProvider;
    this.projectId = data.projectId;

    try {
      const token = await this.accessTokenProvider();
      const wsUrl = new URL(data.url);
      wsUrl.protocol = wsUrl.protocol.replace("http", "ws");
      wsUrl.pathname = `/${data.roomId}`;

      // Add query parameters for authentication
      wsUrl.searchParams.set("user_id", this.doc.clientID.toString());
      wsUrl.searchParams.set("project_id", data.projectId);
      wsUrl.searchParams.set("token", token); // Pass token as query param since we can't set headers

      this.ws = new WebSocket(wsUrl.href);
      this.ws.binaryType = "arraybuffer";

      this.setupWebSocketListeners();
      this.setupDocListeners();

      console.log("Attempting WebSocket connection to:", wsUrl.origin + wsUrl.pathname);
    } catch (error) {
      console.error("Failed to setup WebSocket:", error);
      throw error;
    }
  }

  // Update the reconnect method
  private reconnect() {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
    }

    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      const delay =
        this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);

      this.reconnectTimer = setTimeout(async () => {
        try {
          if (this.ws) {
            const originalUrl = new URL(this.ws.url);
            const baseUrl = `${originalUrl.protocol}//${originalUrl.host}`;
            const roomId = originalUrl.pathname.slice(1);
            await this.setupSocket({
              url: baseUrl,
              roomId,
              projectId: this.projectId || "",
              accessTokenProvider: this.accessTokenProvider || (() => ""),
            });
          }
        } catch (error) {
          console.error("Reconnection failed:", error);
        }
      }, delay);
    }
  }

  private setupWebSocketListeners() {
    this.ws.addEventListener("open", this.onConnectionEstablished);
    this.ws.addEventListener("close", this.onConnectionDisconnect);
    this.ws.addEventListener("error", this.onConnectionError);
    this.ws.addEventListener("message", this.handleMessage);
  }

  private setupDocListeners() {
    this.doc.on("update", this.onDocUpdate);
  }

  protected onConnectionEstablished() {
    this.reconnectAttempts = 0;
    this.socketReady = true;
    this.initializeRoom().catch(console.error);
  }

  protected onConnectionDisconnect() {
    this.socketReady = false;
    this.firstSyncComplete = false;
    this.reconnect();
  }

  protected onConnectionError(error: Event) {
    console.error("WebSocket error:", error);
    this.reconnect();
  }

  protected async onAuthenticateRequest() {
    const token = await this.accessTokenProvider?.();
    if (token && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type: "authenticate", token }));
    }
  }

  protected async handleMessage(event: MessageEvent) {
    try {
      if (event.data instanceof ArrayBuffer) {
        // Handle binary message (Yjs update)
        const update = new Uint8Array(event.data);
        await this.onPeerUpdate({ update });
      } else if (typeof event.data === "string") {
        // Handle text message
        const data = JSON.parse(event.data);
        if (data.type === "authenticate") {
          await this.onAuthenticateRequest();
        } else if (data.type === "ready") {
          this.onReady();
        }
      }
    } catch (error) {
      console.error("Error handling message:", error);
    }
  }

  protected async initializeRoom() {
    try {
      await this.sendFlowMessage({
        event: {
          tag: "Create",
          content: { room_id: this.doc.clientID.toString() },
        },
      });

      await this.sendFlowMessage({
        event: {
          tag: "Join",
          content: { room_id: this.doc.clientID.toString() },
        },
      });

      await this.sendFlowMessage({
        event: {
          tag: "Emit",
          content: { data: "" },
        },
        session_command: {
          tag: "Start",
          content: {
            project_id: this.projectId || "",
            user: {
              id: this.doc.clientID.toString(),
              tenant_id: this.projectId,
              name: "defaultName",
              email: "defaultEmail@example.com",
            },
          },
        },
      });

      await this.syncData();
    } catch (error) {
      console.error("Failed to initialize room:", error);
    }
  }

  async isReady(): Promise<boolean> {
    if (this.socketReady) return true;
    await sleep(100);
    return await this.isReady();
  }

  protected onReady() {
    this.socketReady = true;
  }

  protected onPeerUpdate(data: { update: ArrayBuffer | Uint8Array }) {
    const update = data.update instanceof ArrayBuffer
      ? new Uint8Array(data.update)
      : data.update;
  
    const currentState = Y.encodeStateAsUpdateV2(this.doc);
    const diffUpdate = Y.diffUpdateV2(update, currentState);
    Y.applyUpdateV2(this.doc, diffUpdate, 'peer');
    this.onUpdateHandlers.forEach((handler) => handler(update));
  }

  async syncData() {
    await this.isReady();

    const currentState = Y.encodeStateAsUpdateV2(this.doc);
    const stateVector = Y.encodeStateVectorFromUpdateV2(currentState);
    
    if (this.ws.readyState === WebSocket.OPEN) {
      const syncMessage = createBinaryMessage(MessageType.SYNC, stateVector);
      this.ws.send(syncMessage);
    }

    if (!this.firstSyncComplete) {
      this.firstSyncComplete = true;
      queueMicrotask(() => this.syncData());
    }
  }

  private async sendFlowMessage(message: FlowMessage): Promise<void> {
    return new Promise((resolve, reject) => {
      if (this.ws.readyState !== WebSocket.OPEN) {
        reject(new Error("WebSocket is not connected"));
        return;
      }

      try {
        this.ws.send(JSON.stringify(message));
        resolve();
      } catch (error) {
        reject(error);
      }
    });
  }

  protected onDocUpdate(update: Uint8Array, origin: unknown) {
    if (origin === this.doc.clientID && this.ws.readyState === WebSocket.OPEN) {
      const stateVector = Y.encodeStateVectorFromUpdateV2(update);
      const diffUpdate = Y.diffUpdateV2(update, stateVector);
      
      const updateMessage = createBinaryMessage(MessageType.UPDATE, diffUpdate);
      this.ws.send(updateMessage);
    }
  }

  onUpdate(handler: (update: Uint8Array) => void) {
    this.onUpdateHandlers.push(handler);
  }

  destroy() {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
    }

    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.sendFlowMessage({
        event: {
          tag: "Emit",
          content: { data: "" },
        },
        session_command: {
          tag: "End",
          content: {
            project_id: this.projectId || "",
            user: {
              id: this.doc.clientID.toString(),
              tenant_id: this.projectId,
              name: "defaultName",
              email: "defaultEmail@example.com",
            },
          },
        },
      }).finally(() => {
        this.ws.close();
      });
    }
    this.doc.destroy();
  }
}
