import CustomNode, { type CustomNodeProps } from "./CustomNode";

// selected style ???
// className="bg-[#631628] border border-[#915b68] rounded-sm pl-1 w-[150px] h-[50px]"

const TransformerNode: React.FC<CustomNodeProps> = props => {
  // const onChange = useCallback(
  //   (evt: any) => {
  //     console.log("EVT", evt.target.value);
  //     console.log("data", data);
  //   },
  //   [data],
  // );
  return <CustomNode className="bg-[#631628] rounded-sm pl-1 w-[150px] h-[50px]" {...props} />;
};

export default TransformerNode;
