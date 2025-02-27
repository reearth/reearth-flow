import { Array as YArray } from "yjs";

import Canvas from "@flow/features/Canvas";
import { YWorkflow } from "@flow/lib/yjs/types";

type Props = {
  yWorkflows: YArray<YWorkflow>;
};

const SharedCanvas: React.FC<Props> = ({ yWorkflows }) => {
  console.log("yWorkflows", yWorkflows);
  return (
    <div>
      <Canvas
        nodes={nodes}
        edges={edges}
        canvasLock={!!locallyLockedNode}
        onWorkflowAdd={handleWorkflowAdd}
        onNodesAdd={handleNodesAdd}
        onNodesChange={handleNodesChange}
        onNodeHover={handleNodeHover}
        onNodeDoubleClick={handleNodeDoubleClick}
        onNodePickerOpen={handleNodePickerOpen}
        onEdgesAdd={handleEdgesAdd}
        onEdgesChange={handleEdgesChange}
        onEdgeHover={handleEdgeHover}
      />
    </div>
  );
};

export default SharedCanvas;
