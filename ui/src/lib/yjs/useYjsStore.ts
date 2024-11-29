import { useCallback, useEffect, useRef, useState } from "react";
import { useY } from "react-yjs";
import { WebsocketProvider } from "y-websocket";
import * as Y from "yjs";

import { config } from "@flow/config";
import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useCurrentProject } from "@flow/stores";
import type { Edge, Node, Workflow } from "@flow/types";
import { isDefined } from "@flow/utils";
import { createWorkflowsYaml } from "@flow/utils/engineWorkflowYaml/workflowYaml";

import { useDeployment } from "../gql/deployment";
import { useT } from "../i18n";

import useWorkflowTabs from "./useWorkflowTabs";
import useYEdge from "./useYEdge";
import useYNode from "./useYNode";
import useYWorkflow from "./useYWorkflow";
import { yWorkflowBuilder, type YWorkflow } from "./utils";

export default ({
  workflowId,
  handleWorkflowIdChange,
}: {
  workflowId?: string;
  handleWorkflowIdChange: (id?: string) => void;
}) => {
  const { toast } = useToast();
  const t = useT();

  const [currentProject] = useCurrentProject();
  const { createDeployment, useUpdateDeployment } = useDeployment();

  const yWebSocketRef = useRef<WebsocketProvider | null>(null);
  useEffect(() => () => yWebSocketRef.current?.destroy(), []);

  const [undoManager, setUndoManager] = useState<Y.UndoManager | null>(null);

  const [{ yWorkflows, currentUserClientId, undoTrackerActionWrapper }] =
    useState(() => {
      const yDoc = new Y.Doc();
      const { websocket, websocketToken } = config();
      if (workflowId && websocket && websocketToken) {
        yWebSocketRef.current = new WebsocketProvider(
          websocket,
          workflowId,
          yDoc,
          { params: { token: websocketToken } },
        );
      }

      const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
      const yWorkflow = yWorkflowBuilder("main", "Main Workflow");
      yWorkflows.push([yWorkflow]);

      const currentUserClientId = yDoc.clientID;

      // NOTE: any changes to the yDoc should be wrapped in a transact
      const undoTrackerActionWrapper = (callback: () => void) =>
        yDoc.transact(callback, currentUserClientId);

      return { yWorkflows, currentUserClientId, undoTrackerActionWrapper };
    });

  useEffect(() => {
    if (yWorkflows) {
      const manager = new Y.UndoManager(yWorkflows, {
        trackedOrigins: new Set([currentUserClientId]), // Only track local changes
        captureTimeout: 200, // default is 500. 200ms is a good balance between performance and user experience
      });
      setUndoManager(manager);

      return () => {
        manager.destroy(); // Clean up UndoManager on component unmount
      };
    }
  }, [yWorkflows, currentUserClientId]);

  const handleWorkflowUndo = useCallback(() => {
    if (undoManager?.undoStack && undoManager.undoStack.length > 0) {
      undoManager?.undo();
    }
  }, [undoManager]);

  const handleWorkflowRedo = useCallback(() => {
    if (undoManager?.redoStack && undoManager.redoStack.length > 0) {
      undoManager?.redo();
    }
  }, [undoManager]);

  const rawWorkflows = useY(yWorkflows);

  const {
    workflows,
    openWorkflows,
    currentWorkflowIndex,
    setWorkflows,
    setOpenWorkflowIds,
    handleWorkflowOpen,
    handleWorkflowClose,
  } = useWorkflowTabs({ workflowId, rawWorkflows, handleWorkflowIdChange });

  const {
    currentYWorkflow,
    handleWorkflowAdd,
    handleWorkflowsRemove,
    handleWorkflowRename,
  } = useYWorkflow({
    yWorkflows,
    workflows,
    currentWorkflowIndex,
    undoTrackerActionWrapper,
    setWorkflows,
    setOpenWorkflowIds,
    handleWorkflowIdChange,
    handleWorkflowOpen,
  });

  const nodes = useY(
    currentYWorkflow?.get("nodes") ?? new Y.Array<Node>(),
  ) as Node[];
  const edges = useY(
    currentYWorkflow?.get("edges") ?? new Y.Array<Edge>(),
  ) as Edge[];

  const handleWorkflowDeployment = useCallback(
    async (deploymentId?: string, description?: string) => {
      const {
        name: projectName,
        workspaceId,
        id: projectId,
      } = currentProject ?? {};

      if (!workspaceId || !projectId) return;

      const { workflowId, yamlWorkflow } =
        createWorkflowsYaml(
          projectName,
          rawWorkflows
            .map((w): Workflow | undefined => {
              if (!w || w.nodes.length < 1) return undefined;
              const id = w.id as string;
              const name = w.name as string;
              const n = w.nodes as Node[];
              const e = w.edges as Edge[];
              return { id, name, nodes: n, edges: e };
            })
            .filter(isDefined),
        ) ?? {};

      if (!workflowId || !yamlWorkflow) {
        toast({
          title: t("Empty workflow detected"),
          description: t("You cannot create a deployment without a workflow."),
        });
        return;
      }

      if (deploymentId) {
        await useUpdateDeployment(
          deploymentId,
          workflowId,
          yamlWorkflow,
          description,
        );
      } else {
        await createDeployment(
          workspaceId,
          projectId,
          workflowId,
          yamlWorkflow,
          description,
        );
      }
    },
    [
      rawWorkflows,
      currentProject,
      t,
      createDeployment,
      useUpdateDeployment,
      toast,
    ],
  );

  const { handleNodesUpdate } = useYNode({
    currentYWorkflow,
    undoTrackerActionWrapper,
    handleWorkflowsRemove,
  });

  const { handleEdgesUpdate } = useYEdge({
    currentYWorkflow,
    undoTrackerActionWrapper,
  });

  return {
    nodes,
    edges,
    openWorkflows,
    handleWorkflowDeployment,
    handleWorkflowClose,
    handleWorkflowAdd,
    handleNodesUpdate,
    handleEdgesUpdate,
    handleWorkflowUndo,
    handleWorkflowRedo,
    handleWorkflowRename,
  };
};
