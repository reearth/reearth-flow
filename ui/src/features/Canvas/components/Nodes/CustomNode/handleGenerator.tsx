import { Handle, Position } from "reactflow";

import type { NodeType } from "./types";

const handleGenerator = (type: NodeType) => {
  const handleStyle = { top: 10 };
  return type === "reader" ? (
    <>
      <Handle type="source" position={Position.Right} id="main-source" />
      <Handle type="source" position={Position.Right} id="secondary-source" style={handleStyle} />
    </>
  ) : type === "writer" ? (
    <Handle id="target" type="target" position={Position.Left} />
  ) : type === "transformer" ? (
    <>
      <Handle id="target" type="target" position={Position.Left} />
      <Handle type="source" position={Position.Right} id="main-source" />
      <Handle type="source" position={Position.Right} id="secondary-source" style={handleStyle} />
    </>
  ) : null;
};

export default handleGenerator;
