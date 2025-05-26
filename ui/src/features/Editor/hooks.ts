import { useReactFlow } from "@xyflow/react";
import {
  MouseEvent,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { useY } from "react-yjs";
import { Map as YMap, UndoManager as YUndoManager } from "yjs";

import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { useShortcuts } from "@flow/hooks";
import { useSharedProject } from "@flow/lib/gql";
import { checkForReader } from "@flow/lib/reactFlow";
import { useYjsStore } from "@flow/lib/yjs";
import type { YWorkflow } from "@flow/lib/yjs/types";
import useWorkflowTabs from "@flow/lib/yjs/useWorkflowTabs";
import { useCurrentProject } from "@flow/stores";
import type { Algorithm, Direction, Edge, Node } from "@flow/types";

import useCanvasCopyPaste from "./useCanvasCopyPaste";
import useDebugRun from "./useDebugRun";
import useDeployment from "./useDeployment";
import useNodeLocker from "./useNodeLocker";
import useUIState from "./useUIState";

export default ({
  yWorkflows,
  undoManager,
  undoTrackerActionWrapper,
}: {
  yWorkflows: YMap<YWorkflow>;
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

  const isSubworkflow = useMemo(() => {
    if (!currentYWorkflow) return false;
    const workflowId = currentYWorkflow.get("id")?.toJSON();
    return workflowId !== DEFAULT_ENTRY_GRAPH_ID;
  }, [currentYWorkflow]);

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
      Object.values(rawEdges).map((edge) => ({
        ...edge,
        selected:
          selectedEdgeIds.includes(edge.id) && !edge.selected
            ? true
            : (edge.selected ?? false),
      })),
    [rawEdges, selectedEdgeIds],
  );

  const hasReader = checkForReader(nodes);

  const { lockedNodeIds, locallyLockedNode, handleNodeLocking } = useNodeLocker(
    { nodes, selectedNodeIds, setSelectedNodeIds },
  );

  const {
    openWorkflows,
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

  // Passed to editor context so needs to be a ref
  const handleNodeDoubleClickRef =
    useRef<(e: MouseEvent | undefined, nodeId: string) => void>(undefined);
  handleNodeDoubleClickRef.current = (
    _e: MouseEvent | undefined,
    nodeId: string,
  ) => {
    handleNodeLocking(nodeId);
  };
  const handleNodeDoubleClick = useCallback(
    (e: MouseEvent | undefined, nodeId: string) =>
      handleNodeDoubleClickRef.current?.(e, nodeId),
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
    hoveredDetails,
    handleNodeHover,
    handleEdgeHover,
    handleNodePickerOpen,
    handleNodePickerClose,
    handleRightPanelOpen,
  } = useUIState({ hasReader });

  const { allowedToDeploy, handleWorkflowDeployment } = useDeployment({
    currentNodes: nodes,
    yWorkflows,
  });

  const { shareProject, unshareProject } = useSharedProject();

  const [currentProject] = useCurrentProject();

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
      keyBinding: { key: "x", commandKey: true },
      callback: () => {
        handleCut(true);
      },
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

  const handleWorkflowRename = useCallback(
    (id: string, newName: string) => {
      handleYWorkflowRename(id, newName);
      setWorkflowsNames((prevNames) =>
        prevNames.map((w) => (w.id === id ? { ...w, name: newName } : w)),
      );
    },
    [handleYWorkflowRename, setWorkflowsNames],
  );

  return {
    isSubworkflow,
    currentWorkflowId,
    openWorkflows,
    currentProject,
    nodes,
    edges,
    selectedEdgeIds,
    lockedNodeIds,
    locallyLockedNode,
    hoveredDetails,
    nodePickerOpen,
    allowedToDeploy,
    rightPanelContent,
    canUndo,
    canRedo,
    isMainWorkflow,
    hasReader,
    handleRightPanelOpen,
    handleWorkflowAdd: handleYWorkflowAdd,
    handleWorkflowDeployment,
    handleProjectShare,
    handleWorkflowOpen,
    handleWorkflowClose,
    handleWorkflowChange: handleCurrentWorkflowIdChange,
    handleWorkflowRedo: handleYWorkflowRedo,
    handleWorkflowUndo: handleYWorkflowUndo,
    handleWorkflowRename,
    handleLayoutChange,
    handleNodesAdd: handleYNodesAdd,
    handleNodesChange: handleYNodesChange,
    handleNodeHover,
    handleNodeDataUpdate: handleYNodeDataUpdate,
    handleNodeDoubleClick,
    handleNodePickerOpen,
    handleNodePickerClose,
    handleEdgesAdd: handleYEdgesAdd,
    handleEdgesChange: handleYEdgesChange,
    handleEdgeHover,
    handleDebugRunStart,
    handleDebugRunStop,
    handleCopy,
    handleCut,
    handlePaste,
  };
};
