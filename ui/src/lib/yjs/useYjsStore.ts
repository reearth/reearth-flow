import { useCallback, useMemo } from "react";
import { useY } from "react-yjs";
import * as Y from "yjs";

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
import { YWorkflow } from "./utils";

export default ({
  workflowId,
  yWorkflows,
  undoManager,
  undoTrackerActionWrapper,
  handleWorkflowIdChange,
}: {
  workflowId?: string;
  yWorkflows: Y.Array<YWorkflow>;
  undoManager: Y.UndoManager | null;
  undoTrackerActionWrapper: (callback: () => void) => void;
  handleWorkflowIdChange: (id?: string) => void;
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
    rawWorkflows,
    currentWorkflowIndex,
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

  const { handleNodesUpdate, handleNodeParamsUpdate } = useYNode({
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
    selectedNodes,
    handleWorkflowDeployment,
    handleWorkflowOpen,
    handleWorkflowClose,
    handleWorkflowAdd,
    handleNodesUpdate,
    handleNodeParamsUpdate,
    handleEdgesUpdate,
    handleWorkflowUndo,
    handleWorkflowRedo,
    canUndo,
    canRedo,
    handleWorkflowRename,
  };
};
