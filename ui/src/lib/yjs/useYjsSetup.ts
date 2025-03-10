import { useEffect, useState } from "react";
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

  const [state, setState] = useState<{
    yDoc: Y.Doc;
    yWorkflows: Y.Array<YWorkflow>;
    undoTrackerActionWrapper: (callback: () => void) => void;
  } | null>(null);
  const [isSynced, setIsSynced] = useState(false);

  useEffect(() => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getArray<YWorkflow>("workflows");

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

        yWebSocketProvider.once("sync", () => {
          const metaMap = yDoc.getMap<boolean>("__meta__");
          
          const mainWorkflowCreated = metaMap.get("mainWorkflowCreated");
          
          if (!mainWorkflowCreated && yWorkflows.length === 0) {
            setTimeout(() => {
              if (!metaMap.get("mainWorkflowCreated") && yWorkflows.length === 0) {
                yDoc.transact(() => {
                  metaMap.set("mainWorkflowCreated", true);
                  
                  const yWorkflow = yWorkflowConstructor(
                    DEFAULT_ENTRY_GRAPH_ID,
                    "Main Workflow",
                  );
                  yWorkflows.insert(0, [yWorkflow]);
                  
                  console.log("Created main workflow with client ID:", yDoc.clientID);
                });
              }
            }, 500);
          }
          
          setIsSynced(true);
        });
      })();
    }

    setState({
      yDoc,
      yWorkflows,
      undoTrackerActionWrapper: (callback: () => void) =>
        yDoc.transact(callback, yDoc.clientID),
    });

    return () => {
      setIsSynced(false);
      yWebSocketProvider?.destroy();
    };
  }, [projectId, workflowId, isProtected, getAccessToken]);

  const { yDoc, yWorkflows, undoTrackerActionWrapper } = state || {};

  const currentUserClientId = yDoc?.clientID;

  useEffect(() => {
    if (yWorkflows) {
      const manager = new Y.UndoManager(yWorkflows, {
        trackedOrigins: new Set([currentUserClientId]),
        captureTimeout: 200,
      });
      setUndoManager(manager);

      return () => {
        manager.destroy();
      };
    }
  }, [yWorkflows, currentUserClientId]);

  useEffect(() => {
    if (yDoc && yWorkflows && isSynced) {
      const observer = () => {
        const metaMap = yDoc.getMap<boolean>("__meta__");
        
        if (yWorkflows.length > 0 && !metaMap.get("mainWorkflowCreated")) {
          yDoc.transact(() => {
            metaMap.set("mainWorkflowCreated", true);
          });
        }
        
        if (metaMap.get("mainWorkflowCreated") && yWorkflows.length === 0) {
          setTimeout(() => {
            if (yWorkflows.length === 0) {
              console.warn("Flag set but no workflows exist, creating main workflow");
              yDoc.transact(() => {
                const yWorkflow = yWorkflowConstructor(
                  DEFAULT_ENTRY_GRAPH_ID,
                  "Main Workflow",
                );
                yWorkflows.insert(0, [yWorkflow]);
              });
            }
          }, 1000);
        }
      };
      
      yWorkflows.observe(observer);
      
      observer();
      
      return () => {
        yWorkflows.unobserve(observer);
      };
    }
  }, [yDoc, yWorkflows, isSynced]);

  return {
    state,
    isSynced,
    undoManager,
    undoTrackerActionWrapper,
  };
};
