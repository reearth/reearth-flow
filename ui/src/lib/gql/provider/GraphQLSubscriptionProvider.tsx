import { createClient, Client } from "graphql-ws";
import {
  createContext,
  useState,
  ReactNode,
  useEffect,
  useContext,
} from "react";

import { config } from "@flow/config";

// Create context for the WebSocket client
const WebSocketContext = createContext<Client | undefined>(undefined);

export const useWsClient = () => {
  const wsClient = useContext(WebSocketContext);
  if (!wsClient) {
    throw new Error("useWsClient must be used within a WebSocketProvider");
  }
  return wsClient;
};

export const GraphQLSubscriptionProvider = ({
  accessToken,
  children,
}: {
  accessToken?: string;
  children?: ReactNode;
}) => {
  const [wsClient, setWsClient] = useState<Client | undefined>();
  const api = config().api;

  useEffect(() => {
    const client = createClient({
      url: `${api}/api/graphql`,
      retryAttempts: 5,
      shouldRetry: () => true,
      connectionParams: () => {
        return {
          headers: {
            authorization: accessToken ? `Bearer ${accessToken}` : "",
          },
        };
      },
    });

    setWsClient(client);

    // Cleanup function to close connection when component unmounts
    return () => {
      // Check if the client has a dispose method
      if (client && typeof (client as any).dispose === "function") {
        (client as any).dispose();
      }
    };
  }, [api, accessToken]);

  return wsClient ? (
    <WebSocketContext.Provider value={wsClient}>
      {children}
    </WebSocketContext.Provider>
  ) : null;
};
