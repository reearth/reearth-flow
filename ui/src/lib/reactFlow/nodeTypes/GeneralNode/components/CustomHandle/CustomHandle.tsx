import {
  Handle,
  HandleProps,
  ReactFlowState,
  getConnectedEdges,
  useNodeId,
  useStore,
} from "@xyflow/react";
import { memo, useMemo } from "react";

import { isExistingConnection } from "@flow/lib/reactFlow/utils";

const selector = (s: ReactFlowState) => ({
  nodeLookup: s.nodeLookup,
  edges: s.edges,
});

type Props = Omit<HandleProps, "isConnectable"> & {
  className?: string;
  isConnectable?: number;
};

const CustomHandle: React.FC<Props> = ({ className, ...props }) => {
  const { nodeLookup, edges } = useStore(selector);
  const nodeId = useNodeId();
  const isValidConnection = useMemo(() => isExistingConnection(edges), [edges]);
  const isHandleConnectable = useMemo(() => {
    if (nodeId && props.isConnectable) {
      const node = nodeLookup.get(nodeId);
      if (!node) return false;
      const connectedEdges = getConnectedEdges([node], edges);

      return connectedEdges.length < props.isConnectable;
    }
  }, [nodeLookup, edges, nodeId, props.isConnectable]);

  return (
    <Handle
      {...props}
      isValidConnection={isValidConnection}
      isConnectable={isHandleConnectable}
      className={`h-full border-none bg-transparent ${className}`}
    />
  );
};

export default memo(CustomHandle);
