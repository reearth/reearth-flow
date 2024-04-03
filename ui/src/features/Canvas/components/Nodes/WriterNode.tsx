import GeneralNode, { type GeneralNodeProps } from "./GeneralNode";

// selected style: border border-[#91855b]

const WriterNode: React.FC<GeneralNodeProps> = props => {
  // const onChange = useCallback(
  //   (evt: any) => {
  //     console.log("EVT", evt.target.value);
  //     console.log("data", data);
  //   },
  //   [data],
  // );
  return <GeneralNode className="bg-[#635116]" {...props} />;
};

export default WriterNode;
