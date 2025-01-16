import { useCallback, useMemo } from "react";
import { useY } from "react-yjs";
import * as Y from "yjs";

import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useCurrentProject } from "@flow/stores";
import type { Edge, Node, Workflow } from "@flow/types";
import { isDefined } from "@flow/utils";
import { jsonToFormData } from "@flow/utils/jsonToFormData";
import { createEngineReadyWorkflow } from "@flow/utils/toEngineWorkflowJson/engineReadyWorkflow";

import { useDeployment } from "../gql/deployment";
import { useT } from "../i18n";

import useWorkflowTabs from "./useWorkflowTabs";
import useYEdge from "./useYEdge";
import useYNode from "./useYNode";
import useYWorkflow from "./useYWorkflow";
import { YWorkflow } from "./utils";

export default ({
  currentWorkflowId,
  yWorkflows,
  undoManager,
  undoTrackerActionWrapper,
  handleCurrentWorkflowIdChange,
}: {
  currentWorkflowId: string;
  yWorkflows: Y.Array<YWorkflow>;
  undoManager: Y.UndoManager | null;
  undoTrackerActionWrapper: (callback: () => void) => void;
  handleCurrentWorkflowIdChange: (id?: string) => void;
}) => {
  const { toast } = useToast();
  const t = useT();

  const [currentProject] = useCurrentProject();
  const { createDeployment, useUpdateDeployment } = useDeployment();

  const handleWorkflowUndo = useCallback(() => {
    const stackLength = undoManager?.undoStack?.length ?? 0;
    if (stackLength > 0) {
      undoManager?.undo();
    }
  }, [undoManager]);

  const handleWorkflowRedo = useCallback(() => {
    const stackLength = undoManager?.redoStack?.length ?? 0;
    if (stackLength > 0) {
      undoManager?.redo();
    }
  }, [undoManager]);

  const canUndo = useMemo(() => {
    const stackLength = undoManager?.undoStack?.length ?? 0;
    return stackLength > 0;
  }, [undoManager?.undoStack?.length]);

  const canRedo = useMemo(() => {
    const stackLength = undoManager?.redoStack?.length ?? 0;
    return stackLength > 0;
  }, [undoManager?.redoStack?.length]);

  const rawWorkflows = useY(yWorkflows);

  const {
    openWorkflows,
    setWorkflows,
    setOpenWorkflowIds,
    handleWorkflowOpen,
    handleWorkflowClose,
  } = useWorkflowTabs({
    currentWorkflowId,
    rawWorkflows,
    handleCurrentWorkflowIdChange,
  });

  const {
    currentYWorkflow,
    handleWorkflowAdd,
    handleWorkflowUpdate,
    handleWorkflowsRemove,
    handleWorkflowRename,
  } = useYWorkflow({
    yWorkflows,
    rawWorkflows,
    currentWorkflowId,
    undoTrackerActionWrapper,
    setWorkflows,
    setOpenWorkflowIds,
  });

  const nodes = useY(
    currentYWorkflow.get("nodes") ?? new Y.Array<Node>(),
  ) as Node[];
  const edges = useY(
    currentYWorkflow.get("edges") ?? new Y.Array<Edge>(),
  ) as Edge[];

  const selectedNodes = useMemo(() => nodes.filter((n) => n.selected), [nodes]);

  const handleWorkflowDeployment = useCallback(
    async (deploymentId?: string, description?: string) => {
      const {
        name: projectName,
        workspaceId,
        id: projectId,
      } = currentProject ?? {};

      if (!workspaceId || !projectId) return;

      const engineReadyWorkflow = createEngineReadyWorkflow(
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
      );

      if (!engineReadyWorkflow) {
        toast({
          title: t("Empty workflow detected"),
          description: t("You cannot create a deployment without a workflow."),
        });
        return;
      }

      const formData = jsonToFormData(
        engineReadyWorkflow,
        engineReadyWorkflow.id,
      );

      if (deploymentId) {
        await useUpdateDeployment(
          deploymentId,
          formData.get("file") ?? undefined,
          description,
        );
      } else {
        await createDeployment(
          workspaceId,
          projectId,
          engineReadyWorkflow,
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

  const { handleNodesUpdate, handleNodesChange, handleNodeParamsUpdate } =
    useYNode({
      currentYWorkflow,
      rawWorkflows,
      yWorkflows,
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
    selectedNodes,
    canUndo,
    canRedo,
    rawWorkflows,
    handleWorkflowDeployment,
    handleWorkflowOpen,
    handleWorkflowClose,
    handleWorkflowAdd,
    handleWorkflowUpdate,
    handleNodesUpdate,
    handleNodesChange,
    handleNodeParamsUpdate,
    handleEdgesUpdate,
    handleWorkflowUndo,
    handleWorkflowRedo,
    handleWorkflowRename,
  };
};
