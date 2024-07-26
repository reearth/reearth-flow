import { BottomPanel, Canvas, LeftPanel, OverlayUI, RightPanel } from "./components";
import useHooks from "./hooks";

export default function Editor() {
  const {
    currentWorkflowId,
    workflows,
    nodes,
    edges,
    // lockedNodeIds,
    locallyLockedNode,
    hoveredDetails,
    handleWorkflowAdd,
    handleWorkflowRemove,
    handleWorkflowChange,
    handleNodesUpdate,
    handleNodeHover,
    handleNodeLocking,
    handleEdgesUpdate,
    handleEdgeHover,
  } = useHooks();

  return (
    <div className="flex h-screen flex-col">
      <div className="relative flex flex-1">
        <LeftPanel nodes={nodes} />
        <div className="flex flex-1 flex-col">
          <OverlayUI hoveredDetails={hoveredDetails}>
            <Canvas
              nodes={nodes}
              edges={edges}
              canvasLock={!!locallyLockedNode}
              onNodesUpdate={handleNodesUpdate}
              onNodeHover={handleNodeHover}
              onNodeLocking={handleNodeLocking}
              onEdgesUpdate={handleEdgesUpdate}
              onEdgeHover={handleEdgeHover}
            />
          </OverlayUI>
          <BottomPanel
            currentWorkflowId={currentWorkflowId}
            workflows={workflows}
            onWorkflowAdd={handleWorkflowAdd}
            onWorkflowRemove={handleWorkflowRemove}
            onWorkflowChange={handleWorkflowChange}
          />
        </div>
        <RightPanel selected={locallyLockedNode} />
      </div>
    </div>
  );
}
