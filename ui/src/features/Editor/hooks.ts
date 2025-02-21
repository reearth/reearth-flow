import { useReactFlow } from "@xyflow/react";
import { MouseEvent, useCallback, useEffect, useMemo, useState } from "react";
import { useY } from "react-yjs";
import { Array as YArray, UndoManager as YUndoManager } from "yjs";

import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { useShortcuts } from "@flow/hooks";
import { useDeployment } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { checkForReader } from "@flow/lib/reactFlow";
import { useYjsStore } from "@flow/lib/yjs";
import { rebuildWorkflow } from "@flow/lib/yjs/conversions";
import type { YWorkflow } from "@flow/lib/yjs/types";
import useWorkflowTabs from "@flow/lib/yjs/useWorkflowTabs";
import { useCurrentProject } from "@flow/stores";
import type { Algorithm, Direction, Edge, Node } from "@flow/types";
import { isDefined } from "@flow/utils";
import { jsonToFormData } from "@flow/utils/jsonToFormData";
import { createEngineReadyWorkflow } from "@flow/utils/toEngineWorkflow/engineReadyWorkflow";

import { useToast } from "../NotificationSystem/useToast";

import useCanvasCopyPaste from "./useCanvasCopyPaste";
import useHover from "./useHover";
import useNodeLocker from "./useNodeLocker";
import useUIState from "./useUIState";

export default ({
  yWorkflows,
  undoManager,
  undoTrackerActionWrapper,
}: {
  yWorkflows: YArray<YWorkflow>;
  undoManager: YUndoManager | null;
  undoTrackerActionWrapper: (callback: () => void) => void;
}) => {
  const { toast } = useToast();
  const { fitView } = useReactFlow();
  const t = useT();

  const [currentProject] = useCurrentProject();
  const { createDeployment, useUpdateDeployment } = useDeployment();

  const [currentWorkflowId, setCurrentWorkflowId] = useState<string>(
    DEFAULT_ENTRY_GRAPH_ID,
  );

  const [selectedNodeIds, setSelectedNodeIds] = useState<string[]>([]);
  const [selectedEdgeIds, setSelectedEdgeIds] = useState<string[]>([]);

  // TODO: If we split canvas more, or use refs, etc, this will become unnecessary @KaWaite
  useEffect(() => {
    fitView({ padding: 0.5 });
  }, [currentWorkflowId, fitView]);

  const {
    canUndo,
    canRedo,
    rawWorkflows,
    currentYWorkflow,
    handleYWorkflowAdd,
    // handleYWorkflowAddFromSelection,
    handleYWorkflowUpdate,
    handleYNodesAdd,
    handleYNodesChange,
    handleYNodeParamsUpdate,
    handleYEdgesAdd,
    handleYEdgesChange,
    handleYWorkflowUndo,
    handleYWorkflowRedo,
    handleYWorkflowRename,
    handleYLayoutChange,
  } = useYjsStore({
    currentWorkflowId,
    yWorkflows,
    undoManager,
    setSelectedNodeIds,
    setSelectedEdgeIds,
    undoTrackerActionWrapper,
  });

  const rawNodes = useY(
    currentYWorkflow.get("nodes") ?? new YArray(),
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

  const rawEdges = useY(
    currentYWorkflow.get("edges") ?? new YArray(),
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

  const allowedToDeploy = useMemo(() => nodes.length > 0, [nodes]);

  const hasReader = checkForReader(nodes);

  const { lockedNodeIds, locallyLockedNode, handleNodeLocking } = useNodeLocker(
    { selectedNodeIds, nodes },
  );

  const {
    openWorkflows,
    isMainWorkflow,
    handleWorkflowOpen,
    handleWorkflowClose,
    handleCurrentWorkflowIdChange,
  } = useWorkflowTabs({
    currentWorkflowId,
    rawWorkflows,
    setCurrentWorkflowId,
  });

  const handleNodeDoubleClick = useCallback(
    (_e: MouseEvent | undefined, node: Node) => {
      if (node.type === "subworkflow" && node.data.subworkflowId) {
        handleWorkflowOpen(node.data.subworkflowId);
      } else {
        fitView({
          nodes: [{ id: node.id }],
          duration: 500,
          padding: 2,
        });
        handleNodeLocking(node.id);
      }
    },
    [handleWorkflowOpen, fitView, handleNodeLocking],
  );

  const { handleCopy, handlePaste } = useCanvasCopyPaste({
    nodes,
    edges,
    rawWorkflows,
    handleWorkflowUpdate: handleYWorkflowUpdate,
    handleNodesAdd: handleYNodesAdd,
    handleNodesChange: handleYNodesChange,
    handleEdgesAdd: handleYEdgesAdd,
  });

  const [openPanel, setOpenPanel] = useState<
    "left" | "right" | "bottom" | undefined
  >(undefined);

  const handlePanelOpen = useCallback(
    (panel?: "left" | "right" | "bottom") => {
      if (!panel || openPanel === panel) {
        setOpenPanel(undefined);
      } else {
        setOpenPanel(panel);
      }
    },
    [openPanel],
  );

  const {
    nodePickerOpen,
    rightPanelContent,
    handleNodePickerOpen,
    handleNodePickerClose,
    handleRightPanelOpen,
  } = useUIState({ hasReader });

  const { hoveredDetails, handleNodeHover, handleEdgeHover } = useHover();

  const handleWorkflowDeployment = useCallback(
    async (description: string, deploymentId?: string) => {
      const {
        name: projectName,
        workspaceId,
        id: projectId,
      } = currentProject ?? {};

      if (!workspaceId || !projectId) return;

      const engineReadyWorkflow = createEngineReadyWorkflow(
        projectName,
        yWorkflows.map((w) => rebuildWorkflow(w)).filter(isDefined),
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

  const handleLayoutChange = useCallback(
    async (algorithm: Algorithm, direction: Direction, _spacing: number) => {
      // We need to wait for the layout to finish before fitting the view
      await Promise.resolve(
        handleYLayoutChange(algorithm, direction, _spacing),
      );
      fitView();
    },
    [fitView, handleYLayoutChange],
  );

  useShortcuts([
    {
      keyBinding: { key: "r", commandKey: false },
      callback: () =>
        handleNodePickerOpen({ x: 0, y: 0 }, "reader", isMainWorkflow),
    },
    {
      keyBinding: { key: "t", commandKey: false },
      callback: () => handleNodePickerOpen({ x: 0, y: 0 }, "transformer"),
    },
    {
      keyBinding: { key: "w", commandKey: false },
      callback: () =>
        handleNodePickerOpen({ x: 0, y: 0 }, "writer", isMainWorkflow),
    },
    {
      keyBinding: { key: "c", commandKey: true },
      callback: handleCopy,
    },
    {
      keyBinding: { key: "v", commandKey: true },
      callback: handlePaste,
    },
    {
      keyBinding: { key: "z", commandKey: true, shiftKey: true },
      callback: handleYWorkflowRedo,
    },
    {
      keyBinding: { key: "z", commandKey: true },
      callback: handleYWorkflowUndo,
    },
    // {
    //   keyBinding: { key: "s", commandKey: false },
    //   callback: () => handleYWorkflowAddFromSelection(nodes, edges),
    // },
  ]);

  return {
    currentWorkflowId,
    openWorkflows,
    nodes,
    edges,
    lockedNodeIds,
    locallyLockedNode,
    hoveredDetails,
    nodePickerOpen,
    openPanel,
    allowedToDeploy,
    rightPanelContent,
    canUndo,
    canRedo,
    isMainWorkflow,
    hasReader,
    handleRightPanelOpen,
    handleWorkflowAdd: handleYWorkflowAdd,
    handleWorkflowDeployment,
    handlePanelOpen,
    handleWorkflowClose,
    handleWorkflowChange: handleCurrentWorkflowIdChange,
    handleWorkflowRedo: handleYWorkflowRedo,
    handleWorkflowUndo: handleYWorkflowUndo,
    handleWorkflowRename: handleYWorkflowRename,
    handleLayoutChange,
    handleNodesAdd: handleYNodesAdd,
    handleNodesChange: handleYNodesChange,
    handleNodeHover,
    handleNodeParamsUpdate: handleYNodeParamsUpdate,
    handleNodeDoubleClick,
    handleNodePickerOpen,
    handleNodePickerClose,
    handleEdgesAdd: handleYEdgesAdd,
    handleEdgesChange: handleYEdgesChange,
    handleEdgeHover,
  };
};
