import { useCallback, useEffect, useRef, useState } from "react";
import { useY } from "react-yjs";
import * as Y from "yjs";

import { config } from "@flow/config";
import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useCurrentProject } from "@flow/stores";
import type { Edge, Node, Workflow } from "@flow/types";
import { isDefined } from "@flow/utils";
import { createWorkflowsYaml } from "@flow/utils/engineWorkflowYaml/workflowYaml";

import { useDeployment } from "../gql/deployment";
import { useT } from "../i18n";

import { SocketYjsManager } from "./socketYjsManager";
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
  const managerRef = useRef<SocketYjsManager | null>(null);
  const [undoManager, setUndoManager] = useState<Y.UndoManager | null>(null);

  // Initialize store and connect
  const [{ yWorkflows, currentUserClientId }] = useState(() => {
    const doc = new Y.Doc();
    const yWorkflows = doc.getArray<YWorkflow>("workflows");

    // Initialize with default workflow if empty
    if (yWorkflows.length === 0) {
      const yWorkflow = yWorkflowBuilder("main", "Main Workflow");
      yWorkflows.push([yWorkflow]);
    }

    return {
      yWorkflows,
      currentUserClientId: doc.clientID,
    };
  });

  useEffect(() => {
    const { websocket: websocketUrl, websocketToken } = config();
    if (!websocketUrl || !websocketToken || !workflowId || !currentProject?.id)
      return;

    const doc = managerRef.current?.getDoc() || yWorkflows.doc;

    // Create and setup socket manager
    if (!doc) return;
    const manager = new SocketYjsManager(doc);
    manager
      .setupSocket({
        url: websocketUrl,
        roomId: workflowId,
        projectId: currentProject.id,
        accessTokenProvider: async () => websocketToken,
      })
      .catch((error) => {
        toast({
          title: t("Connection Error"),
          description: error.message,
          variant: "destructive",
        });
      });

    // Setup undo manager
    const undoMngr = new Y.UndoManager(yWorkflows, {
      trackedOrigins: new Set([currentUserClientId]),
      captureTimeout: 200,
    });
    setUndoManager(undoMngr);

    managerRef.current = manager;

    return () => {
      manager.destroy();
      undoMngr.destroy();
      managerRef.current = null;
    };
  }, [
    workflowId,
    currentProject?.id,
    toast,
    t,
    currentUserClientId,
    yWorkflows,
  ]);

  // Get the raw workflows using useY
  const rawWorkflows = useY(yWorkflows);

  const {
    workflows,
    openWorkflows,
    currentWorkflowIndex,
    setWorkflows,
    setOpenWorkflowIds,
    handleWorkflowOpen,
    handleWorkflowClose,
  } = useWorkflowTabs({
    workflowId,
    rawWorkflows: rawWorkflows as Record<string, string | Node[] | Edge[]>[],
    handleWorkflowIdChange,
  });

  // Create wrapper for undoTrackerActionWrapper
  const undoTrackerActionWrapper = useCallback(
    (callback: () => void) => {
      const doc = managerRef.current?.getDoc();
      if (!doc) return;
      doc.transact(callback, currentUserClientId);
    },
    [currentUserClientId],
  );

  const { currentYWorkflow, handleWorkflowAdd, handleWorkflowsRemove } =
    useYWorkflow({
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

  const { handleNodesUpdate } = useYNode({
    currentYWorkflow,
    undoTrackerActionWrapper,
    handleWorkflowsRemove,
  });

  const { handleEdgesUpdate } = useYEdge({
    currentYWorkflow,
    undoTrackerActionWrapper,
  });

  const handleWorkflowUndo = useCallback(() => {
    if (undoManager?.undoStack.length) {
      undoManager.undo();
    }
  }, [undoManager]);

  const handleWorkflowRedo = useCallback(() => {
    if (undoManager?.redoStack.length) {
      undoManager.redo();
    }
  }, [undoManager]);

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
              if (!w || (w.nodes as Node[]).length < 1) return undefined;
              return {
                id: w.id as string,
                name: w.name as string,
                nodes: w.nodes as Node[],
                edges: w.edges as Edge[],
              };
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
  };
};
