import {
  Dispatch,
  SetStateAction,
  useCallback,
  useMemo,
  useState,
} from "react";
import { useY } from "react-yjs";
import * as Y from "yjs";

import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useCurrentProject } from "@flow/stores";
import type { Edge, Node } from "@flow/types";
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
import { convertYWorkflowToWorkflow } from "./utils/convertToWorkflow";

export default ({
  currentWorkflowId,
  yWorkflows,
  undoManager,
  selectedNodeIds,
  setSelectedNodeIds,
  undoTrackerActionWrapper,
  handleCurrentWorkflowIdChange,
}: {
  currentWorkflowId: string;
  yWorkflows: Y.Array<YWorkflow>;
  undoManager: Y.UndoManager | null;
  selectedNodeIds: string[];
  setSelectedNodeIds: Dispatch<SetStateAction<string[]>>;
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

  const rawWorkflows = yWorkflows.map((w) => convertYWorkflowToWorkflow(w));

  console.log("rawWorkflows", rawWorkflows);
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
    handleWorkflowAddFromSelection,
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

  const rawNodes = useY(
    currentYWorkflow.get("nodes") ?? new Y.Array(),
  ) as Node[];

  const nodes = useMemo(
    () =>
      rawNodes.map((node) => ({
        ...node,
        selected:
          selectedNodeIds.includes(node.id) && !node.selected
            ? true
            : (node.selected ?? false),
      })),
    [rawNodes, selectedNodeIds],
  );

  const [selectedEdgeIds, setSelectedEdgeIds] = useState<string[]>([]);

  const handleEdgeSelection = useCallback(
    (idsToAdd: string[], idsToDelete: string[]) => {
      setSelectedEdgeIds((seids) => {
        const newIds: string[] = seids.filter(
          (id) => !idsToDelete.includes(id),
        );
        newIds.push(...idsToAdd);
        return newIds;
      });
    },
    [],
  );

  const rawEdges = useY(
    currentYWorkflow.get("edges") ?? new Y.Array(),
  ) as Edge[];

  const edges = useMemo(
    () =>
      rawEdges.map((edge) => ({
        ...edge,
        selected:
          selectedEdgeIds.includes(edge.id) && !edge.selected
            ? true
            : (edge.selected ?? false),
      })),
    [rawEdges, selectedEdgeIds],
  );

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
        yWorkflows.map((w) => convertYWorkflowToWorkflow(w)).filter(isDefined),
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
      yWorkflows,
      currentProject,
      t,
      createDeployment,
      useUpdateDeployment,
      toast,
    ],
  );

  const { handleYNodesAdd, handleYNodesChange, handleNodeParamsUpdate } =
    useYNode({
      currentYWorkflow,
      rawWorkflows,
      yWorkflows,
      setSelectedNodeIds,
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
    selectedNodeIds,
    canUndo,
    canRedo,
    rawWorkflows,
    handleWorkflowDeployment,
    handleWorkflowOpen,
    handleWorkflowClose,
    handleWorkflowAdd,
    handleWorkflowAddFromSelection,
    handleWorkflowUpdate,
    handleYNodesAdd,
    handleYNodesChange,
    handleNodeParamsUpdate,
    handleEdgeSelection,
    handleEdgesUpdate,
    handleWorkflowUndo,
    handleWorkflowRedo,
    handleWorkflowRename,
    setOpenWorkflowIds,
    setWorkflows,
  };
};
