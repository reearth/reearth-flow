import { Handle, NodeProps, Position } from "reactflow";

type CustomNodeData = {
  label: string;
};

export type CustomNodeProps = NodeProps<CustomNodeData>;

const TransformerNode: React.FC<CustomNodeProps> = ({ data }) => {
  console.log("D", data);
  // const onChange = useCallback(
  //   (evt: any) => {
  //     console.log("EVT", evt.target.value);
  //     console.log("data", data);
  //   },
  //   [data],
  // );
  const handleStyle = { top: 10 };
  return (
    <>
      <div className="bg-[#631628] border border-[#915b68] rounded-sm pl-1 w-[150px] h-[50px]">
        <label htmlFor="text" className="text-xs">
          {data.label}
        </label>
        {/* <input id="text" name="text" onChange={onChange} className="nodrag" /> */}
      </div>
      <Handle id="target" type="target" position={Position.Left} />
      <Handle type="source" position={Position.Right} id="main-source" />
      <Handle type="source" position={Position.Right} id="secondary-source" style={handleStyle} />
    </>
  );
};

export default TransformerNode;
