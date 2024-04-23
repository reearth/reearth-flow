import GeneralNode, { type GeneralNodeProps } from "./GeneralNode";

const ReaderNode: React.FC<GeneralNodeProps> = props => {
  // const onChange = useCallback(
  //   (evt: any) => {
  //     console.log("EVT", evt.target.value);
  //     console.log("data", data);
  //   },
  //   [data],
  // );
  return <GeneralNode className="bg-[#164E63]/60" {...props} />;
};

export default ReaderNode;
