import GeneralNode, { type GeneralNodeProps } from "./GeneralNode";

// selected style ???
// className="bg-[#631628] border border-[#915b68] rounded-sm pl-1 w-[150px] h-[50px]"

const TransformerNode: React.FC<GeneralNodeProps> = props => {
  // const onChange = useCallback(
  //   (evt: any) => {
  //     console.log("EVT", evt.target.value);
  //     console.log("data", data);
  //   },
  //   [data],
  // );
  return <GeneralNode className="bg-[#631628]" {...props} />;
};

export default TransformerNode;
