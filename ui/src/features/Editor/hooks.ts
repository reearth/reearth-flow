import { useReactFlow } from "@xyflow/react";
import {
  MouseEvent,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { useHotkeys } from "react-hotkeys-hook";
import { useY } from "react-yjs";
import type { Awareness } from "y-protocols/awareness";
import { Doc, Map as YMap, UndoManager as YUndoManager } from "yjs";

import {
  DEFAULT_ENTRY_GRAPH_ID,
  EDITOR_HOT_KEYS,
} from "@flow/global-constants";
import { useProjectExport, useProjectSave } from "@flow/hooks";
import { useSharedProject } from "@flow/lib/gql";
import {
  useAwarenessCursor,
  useSpotlightUser,
  useYjsStore,
} from "@flow/lib/yjs";
import type { YWorkflow } from "@flow/lib/yjs/types";
import useWorkflowTabs from "@flow/lib/yjs/useWorkflowTabs";
import { useCurrentProject } from "@flow/stores";
import type { Algorithm, Direction, Edge, Node } from "@flow/types";

import useCanvasCopyPaste from "./useCanvasCopyPaste";
import useDebugRun from "./useDebugRun";
import useDeployment from "./useDeployment";
import useUIState from "./useUIState";

export default ({
  yDoc,
  yWorkflows,
  yAwareness,
  undoManager,
  undoTrackerActionWrapper,
}: {
  yDoc: Doc | null;
  yWorkflows: YMap<YWorkflow>;
  yAwareness: Awareness;
  undoManager: YUndoManager | null;
  undoTrackerActionWrapper: (
    callback: () => void,
    originPrepend?: string,
  ) => void;
}) => {
  const { fitView } = useReactFlow();

  const [currentWorkflowId, setCurrentWorkflowId] = useState<string>(
    DEFAULT_ENTRY_GRAPH_ID,
  );

  const [selectedNodeIds, setSelectedNodeIds] = useState<string[]>([]);
  const [selectedEdgeIds, setSelectedEdgeIds] = useState<string[]>([]);

  const [openNode, setOpenNode] = useState<Node | undefined>(undefined);

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
    handleYNodeDataUpdate,
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

  const { handleProjectExport } = useProjectExport();

  const { shareProject, unshareProject } = useSharedProject();

  const [currentProject] = useCurrentProject();

  const { handleProjectSnapshotSave, isSaving } = useProjectSave({
    projectId: currentProject?.id,
  });

  const [showBeforeDeleteDialog, setShowBeforeDeleteDialog] =
    useState<boolean>(false);
  const deferredDeleteRef = useRef<{
    resolve: (val: boolean) => void;
  } | null>(null);

  const rawNodes = useY(currentYWorkflow?.get("nodes") ?? new YMap()) as Record<
    string,
    Node
  >;

  // Non-persistant state needs to be managed here
  const nodes = useMemo(
    () =>
      Object.values(rawNodes)
        .map((node) => ({
          ...node,
          selected:
            selectedNodeIds.includes(node.id) && !node.selected
              ? true
              : (node.selected ?? false),
        }))
        .sort((a, b) => {
          // React Flow needs batch nodes to be rendered first
          if (a.type === "batch" && b.type !== "batch") return -1;
          if (a.type !== "batch" && b.type === "batch") return 1;
          return 0;
        }),
    [rawNodes, selectedNodeIds],
  );

  const rawEdges = useY(currentYWorkflow?.get("edges") ?? new YMap()) as Record<
    string,
    Edge
  >;

  // Non-persistant state needs to be managed here
  const edges = useMemo(
    () =>
      Object.values(rawEdges).map((edge) => {
        const sourceNode = nodes.find((n) => n.id === edge.source);
        const targetNode = nodes.find((n) => n.id === edge.target);
        const sourceIsCollapsed = sourceNode?.data?.isCollapsed;
        const targetIsCollapsed = targetNode?.data?.isCollapsed;

        return {
          ...edge,
          selected:
            selectedEdgeIds.includes(edge.id) && !edge.selected
              ? true
              : (edge.selected ?? false),
          reconnectable: sourceIsCollapsed || targetIsCollapsed ? false : true,
        };
      }),
    [rawEdges, selectedEdgeIds, nodes],
  );

  const {
    openWorkflows,
    openWorkflowIds,
    isMainWorkflow,
    handleWorkflowOpen,
    handleWorkflowClose,
    handleCurrentWorkflowIdChange,
    setWorkflowsNames,
  } = useWorkflowTabs({
    currentWorkflowId,
    rawWorkflows,
    setCurrentWorkflowId,
  });

  const handleOpenNode = useCallback(
    (nodeId?: string) => {
      if (!nodeId) {
        setOpenNode(undefined);
      }
      setOpenNode((on) =>
        on?.id === nodeId ? undefined : nodes.find((n) => n.id === nodeId),
      );
    },
    [nodes, setOpenNode],
  );

  // Passed to editor context so needs to be a ref
  const handleNodeSettingsClickRef =
    useRef<(e: MouseEvent | undefined, nodeId: string) => void>(undefined);
  handleNodeSettingsClickRef.current = (
    _e: MouseEvent | undefined,
    nodeId: string,
  ) => {
    handleOpenNode(nodeId);
  };

  const handleNodeSettings = useCallback(
    (e: MouseEvent | undefined, nodeId: string) =>
      handleNodeSettingsClickRef.current?.(e, nodeId),
    [],
  );

  const { handleCopy, handleCut, handlePaste } = useCanvasCopyPaste({
    nodes,
    edges,
    rawWorkflows,
    handleWorkflowUpdate: handleYWorkflowUpdate,
    handleNodesAdd: handleYNodesAdd,
    handleNodesChange: handleYNodesChange,
    handleEdgesAdd: handleYEdgesAdd,
    handleEdgesChange: handleYEdgesChange,
  });

  const {
    nodePickerOpen,
    rightPanelContent,
    handleNodePickerOpen,
    handleNodePickerClose,
    handleRightPanelOpen,
  } = useUIState();

  const { allowedToDeploy, handleWorkflowDeployment } = useDeployment({
    currentNodes: nodes,
    yWorkflows,
  });

  const handleProjectShare = useCallback(
    (share: boolean) => {
      if (!currentProject) return;

      if (share) {
        shareProject({
          projectId: currentProject.id,
          workspaceId: currentProject.workspaceId,
        });
      } else {
        unshareProject({
          projectId: currentProject.id,
          workspaceId: currentProject.workspaceId,
        });
      }
    },
    [currentProject, shareProject, unshareProject],
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

  const { handleDebugRunStart, handleDebugRunStop } = useDebugRun({
    rawWorkflows,
  });

  const handleBeforeDeleteNodes = useCallback(
    ({ nodes: nodesToDelete }: { nodes: Node[] }) => {
      return new Promise<boolean>((resolve) => {
        const deletingIds = new Set(nodesToDelete.map((node) => node.id));

        let totalInputRouters = 0;
        let totalOutputRouters = 0;
        let remainingInputRouters = 0;
        let remainingOutputRouters = 0;

        for (const node of nodes) {
          const officalName = node.data.officialName;
          if (officalName !== "InputRouter" && officalName !== "OutputRouter")
            continue;
          const isDeleting = deletingIds.has(node.id);

          if (officalName === "InputRouter") {
            totalInputRouters++;
            if (!isDeleting) remainingInputRouters++;
          } else if (officalName === "OutputRouter") {
            totalOutputRouters++;
            if (!isDeleting) remainingOutputRouters++;
          }
        }

        const isDeletingLastInputRouter =
          totalInputRouters > 0 && remainingInputRouters === 0;

        const isDeletingLastOutputRouter =
          totalOutputRouters > 0 && remainingOutputRouters === 0;

        if (isDeletingLastInputRouter || isDeletingLastOutputRouter) {
          deferredDeleteRef.current = { resolve };
          setShowBeforeDeleteDialog(true);
        } else {
          resolve(true);
        }
      });
    },
    [nodes],
  );

  const handleDeleteDialogClose = () => setShowBeforeDeleteDialog(false);

  useHotkeys(
    EDITOR_HOT_KEYS,
    (event, handler) => {
      const hasModifier = event.metaKey || event.ctrlKey;
      const hasShift = event.shiftKey;

      switch (handler.keys?.join("")) {
        case "s":
          if (hasModifier && !isSaving) handleProjectSnapshotSave?.();
          break;
        case "z":
          if (hasModifier && hasShift) handleYWorkflowRedo?.();
          if (hasModifier && !hasShift) handleYWorkflowUndo?.();
          break;
      }
    },
    { preventDefault: true },
  );

  const handleWorkflowRename = useCallback(
    (id: string, newName: string) => {
      handleYWorkflowRename(id, newName);
      setWorkflowsNames((prevNames) =>
        prevNames.map((w) => (w.id === id ? { ...w, name: newName } : w)),
      );
    },
    [handleYWorkflowRename, setWorkflowsNames],
  );

  const handleCurrentProjectExport = () => {
    if (yDoc && currentProject) {
      handleProjectExport({ yDoc, project: currentProject });
    }
  };

  const { self, users, handlePaneMouseMove } = useAwarenessCursor({
    yAwareness,
  });

  const {
    spotlightUser,
    spotlightUserClientId,
    handleSpotlightUserSelect,
    handleSpotlightUserDeselect,
  } = useSpotlightUser({
    yAwareness,
    users,
    currentWorkflowId,
    openWorkflowIds,
    handleWorkflowOpen,
    handleWorkflowClose,
  });

  return {
    currentWorkflowId,
    currentYWorkflow,
    openWorkflows,
    currentProject,
    self,
    users,
    nodes,
    edges,
    selectedEdgeIds,
    openNode,
    nodePickerOpen,
    allowedToDeploy,
    rightPanelContent,
    canUndo,
    canRedo,
    isMainWorkflow,
    deferredDeleteRef,
    isSaving,
    showBeforeDeleteDialog,
    spotlightUserClientId,
    spotlightUser,
    handleRightPanelOpen,
    handleWorkflowAdd: handleYWorkflowAdd,
    handleWorkflowDeployment,
    handleProjectShare,
    handleCurrentProjectExport,
    handleWorkflowOpen,
    handleWorkflowClose,
    handleWorkflowChange: handleCurrentWorkflowIdChange,
    handleWorkflowRedo: handleYWorkflowRedo,
    handleWorkflowUndo: handleYWorkflowUndo,
    handleWorkflowRename,
    handleLayoutChange,
    handleNodesAdd: handleYNodesAdd,
    handleNodesChange: handleYNodesChange,
    handleBeforeDeleteNodes,
    handleDeleteDialogClose,
    handleNodeDataUpdate: handleYNodeDataUpdate,
    handleOpenNode,
    handleNodeSettings,
    handleNodePickerOpen,
    handleNodePickerClose,
    handleEdgesAdd: handleYEdgesAdd,
    handleEdgesChange: handleYEdgesChange,
    handleDebugRunStart,
    handleDebugRunStop,
    handleCopy,
    handleCut,
    handlePaste,
    handleProjectSnapshotSave,
    handlePaneMouseMove,
    handleSpotlightUserSelect,
    handleSpotlightUserDeselect,
  };
};
