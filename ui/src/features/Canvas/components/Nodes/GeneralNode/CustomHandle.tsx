import { useMemo } from "react";
import {
  Handle,
  HandleProps,
  ReactFlowState,
  getConnectedEdges,
  useNodeId,
  useStore,
} from "reactflow";

const selector = (s: ReactFlowState) => ({
  nodeInternals: s.nodeInternals,
  edges: s.edges,
});

type Props = Omit<HandleProps, "isConnectable"> & {
  isConnectable?: number;
};

const CustomHandle: React.FC<Props> = props => {
  const { nodeInternals, edges } = useStore(selector);
  const nodeId = useNodeId();

  const isHandleConnectable = useMemo(() => {
    if (nodeId && props.isConnectable) {
      const node = nodeInternals.get(nodeId);
      if (!node) return false;
      const connectedEdges = getConnectedEdges([node], edges);

      return connectedEdges.length < props.isConnectable;
    }
  }, [nodeInternals, edges, nodeId, props.isConnectable]);

  return (
    <Handle
      {...props}
      isConnectable={isHandleConnectable}
      className="bg-zinc-300/75 -z-50 border-none rounded-none h-4"
    />
  );
};

export default CustomHandle;
