import CustomNode, { type CustomNodeProps } from "./CustomNode";

const WriterNode: React.FC<CustomNodeProps> = props => {
  // const onChange = useCallback(
  //   (evt: any) => {
  //     console.log("EVT", evt.target.value);
  //     console.log("data", data);
  //   },
  //   [data],
  // );
  return (
    <CustomNode
      className="bg-[#635116] border border-[#91855b] rounded-sm pl-1 w-[150px] h-[50px]"
      {...props}
    />
  );
};

export default WriterNode;
