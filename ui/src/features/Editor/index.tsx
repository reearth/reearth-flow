import {
  BottomPanel,
  Canvas,
  LeftPanel,
  OverlayUI,
  RightPanel,
} from "./components";
import useHooks from "./hooks";

export default function Editor() {
  const {
    currentWorkflowId,
    openWorkflows,
    nodes,
    edges,
    // lockedNodeIds,
    locallyLockedNode,
    hoveredDetails,
    nodePickerOpen,
    openPanel,
    handleDeploymentReadyWorkflows,
    handlePanelOpen,
    handleWorkflowClose,
    handleWorkflowAdd,
    handleWorkflowChange,
    handleNodesUpdate,
    handleNodeHover,
    handleNodeLocking,
    handleNodePickerOpen,
    handleNodePickerClose,
    handleEdgesUpdate,
    handleEdgeHover,
    handleWorkflowRedo,
    handleWorkflowUndo,
  } = useHooks();

  return (
    <div className="flex h-screen flex-col">
      <div className="relative flex flex-1">
        <LeftPanel
          nodes={nodes}
          isOpen={openPanel === "left" && !locallyLockedNode}
          onOpen={handlePanelOpen}
          onNodesChange={handleNodesUpdate}
          onNodeLocking={handleNodeLocking}
        />
        <div className="flex flex-1 flex-col">
          <OverlayUI
            hoveredDetails={hoveredDetails}
            nodePickerOpen={nodePickerOpen}
            nodes={nodes}
            onDeploymentReadyWorkflows={handleDeploymentReadyWorkflows}
            onWorkflowUndo={handleWorkflowUndo}
            onWorkflowRedo={handleWorkflowRedo}
            onNodesChange={handleNodesUpdate}
            onNodeLocking={handleNodeLocking}
            onNodePickerClose={handleNodePickerClose}>
            <Canvas
              nodes={nodes}
              edges={edges}
              canvasLock={!!locallyLockedNode}
              onNodesUpdate={handleNodesUpdate}
              onNodeHover={handleNodeHover}
              onNodeLocking={handleNodeLocking}
              onNodePickerOpen={handleNodePickerOpen}
              onEdgesUpdate={handleEdgesUpdate}
              onEdgeHover={handleEdgeHover}
            />
          </OverlayUI>
          <BottomPanel
            currentWorkflowId={currentWorkflowId}
            openWorkflows={openWorkflows}
            isOpen={openPanel === "bottom" && !locallyLockedNode}
            onOpen={handlePanelOpen}
            onWorkflowClose={handleWorkflowClose}
            onWorkflowAdd={handleWorkflowAdd}
            onWorkflowChange={handleWorkflowChange}
          />
        </div>
        <RightPanel selected={locallyLockedNode} />
      </div>
    </div>
  );
}
