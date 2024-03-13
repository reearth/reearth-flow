import { NodeProps } from "reactflow";

import handleGenerator from "./handleGenerator";
import type { NodePosition, NodeType } from "./types";

type NodeData = {
  name: string;
  position: NodePosition;
};

export type CustomNodeProps = NodeProps<NodeData> & {
  className?: string;
  onHover?: (nodeInfo?: { id: string; type: NodeType; position: NodePosition }) => void;
};

const CustomNode: React.FC<CustomNodeProps> = ({ className, data, type, ...props }) => {
  // console.log("D", data);
  console.log("P", props);
  // const onChange = useCallback(
  //   (evt: any) => {
  //     console.log("EVT", evt.target.value);
  //     console.log("data", data);
  //   },
  //   [data],
  // );
  return (
    <>
      <div className={className}>
        <label htmlFor="text" className="text-xs">
          {data.name}
        </label>
        {/* <input id="text" name="text" onChange={onChange} className="nodrag" /> */}
      </div>
      {handleGenerator(type as NodeType)}
    </>
  );
};

export default CustomNode;
