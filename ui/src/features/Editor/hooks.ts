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
import { Array as YArray, UndoManager as YUndoManager } from "yjs";

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
  yWorkflows: YArray<YWorkflow>;
  undoManager: YUndoManager | null;
  undoTrackerActionWrapper: (callback: () => void) => void;
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

  // Non-persistant state needs to be managed here
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

  // Non-persistant state needs to be managed here
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
  } = useWorkflowTabs({
    currentWorkflowId,
    rawWorkflows,
    setCurrentWorkflowId,
  });

  // Passed to editor context so needs to be a ref
  const handleNodeDoubleClickRef =
    useRef<(e: MouseEvent | undefined, node: Node) => void>(undefined);
  handleNodeDoubleClickRef.current = (
    _e: MouseEvent | undefined,
    node: Node,
  ) => {
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
  };
  const handleNodeDoubleClick = useCallback(
    (e: MouseEvent | undefined, node: Node) =>
      handleNodeDoubleClickRef.current?.(e, node),
    [],
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

  const {
    openPanel,
    nodePickerOpen,
    rightPanelContent,
    hoveredDetails,
    handleNodeHover,
    handleEdgeHover,
    handlePanelOpen,
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

  // console.log("rawWorkflows", rawWorkflows);
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
    handleProjectShare,
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
    handleDebugRunStart,
    handleDebugRunStop,
  };
};
