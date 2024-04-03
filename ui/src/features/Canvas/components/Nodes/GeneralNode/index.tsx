import { NodeProps, Position } from "reactflow";

import { ReaderIcon, TransformerIcon, WriterIcon } from "@flow/components";

import CustomHandle from "./CustomHandle";
import type { NodePosition, NodeType } from "./types";

type NodeData = {
  name: string;
  position: NodePosition;
  inputs?: string[];
  outputs?: string[];
};

export type GeneralNodeProps = NodeProps<NodeData> & {
  className?: string;
  onHover?: (nodeInfo?: { id: string; type: NodeType; position: NodePosition }) => void;
};

const typeIconClasses = "w-[12px] h-[100%]";

const GeneralNode: React.FC<GeneralNodeProps> = ({ className, data, type, ...props }) => {
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
      <div className={`flex relative w-[150px] rounded-sm bg-zinc-800`}>
        <div className={`flex justify-center w-7 rounded-l-sm ${className}`}>
          {type === "reader" ? (
            <ReaderIcon className={typeIconClasses} />
          ) : type === "writer" ? (
            <WriterIcon className={typeIconClasses} />
          ) : type === "transformer" ? (
            <TransformerIcon className={typeIconClasses} />
          ) : null}
        </div>
        <p className="text-xs p-1.5 text-zinc-300 leading-none text-center truncate">{data.name}</p>
        {type !== "reader" && <CustomHandle id="target" type="target" position={Position.Left} />}
      </div>
      <div
        id="handle-wrapper"
        className="absolute bg-zinc-700 border-b border-zinc-900 rounded-b-md ml-auto mr-auto left-0 right-0 w-[90%]">
        {data.outputs?.map((output, index) => (
          <div key={output + index} className="relative border-b border-zinc-900 py-0.5 px-1.5">
            <CustomHandle type="source" position={Position.Right} id={output} />
            <p className="text-xs pr-1 text-end">{output}</p>
          </div>
        ))}
      </div>
    </>
  );
};

export default GeneralNode;
