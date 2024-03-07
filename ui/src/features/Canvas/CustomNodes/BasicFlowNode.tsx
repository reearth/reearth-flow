import { Handle, NodeProps, Position } from "reactflow";

type CustomNodeData = {
  label: string;
};

export type CustomNodeProps = NodeProps<CustomNodeData>;

const BasicFlowNode: React.FC<CustomNodeProps> = ({ data }) => {
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
      <Handle id="target" type="target" position={Position.Left} />
      <div className="bg-cyan-900 text-zinc-300 border border-cyan-700 rounded-sm p-[8px] w-[150px] h-[50px]">
        <label htmlFor="text" className="text-sm">
          {data.label}
        </label>
        {/* <input id="text" name="text" onChange={onChange} className="nodrag" /> */}
      </div>
      <Handle type="source" position={Position.Right} id="main-source" />
      <Handle type="source" position={Position.Right} id="secondary-source" style={handleStyle} />
    </>
  );
};

export default BasicFlowNode;
