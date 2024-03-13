import CustomNode, { type CustomNodeProps } from "./CustomNode";

const ReaderNode: React.FC<CustomNodeProps> = props => {
  // const onChange = useCallback(
  //   (evt: any) => {
  //     console.log("EVT", evt.target.value);
  //     console.log("data", data);
  //   },
  //   [data],
  // );
  return (
    <CustomNode
      className="bg-cyan-900 border border-cyan-700 rounded-sm pl-1 w-[150px] h-[50px]"
      {...props}
    />
  );
};

export default ReaderNode;
