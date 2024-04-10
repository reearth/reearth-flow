import { DiscIcon } from "@radix-ui/react-icons";
import { NodeProps, Position } from "reactflow";

import { ReaderIcon, TransformerIcon } from "@flow/components";

import CustomHandle from "./CustomHandle";
import type { NodePosition, NodeType } from "./types";

type NodeData = {
  name: string;
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
  // const onChange = useCallback(
  //   (evt: any) => {
  //     console.log("EVT", evt.target.value);
  //     console.log("data", data);
  //   },
  //   [data],
  // );
  console.log("props: ", props);

  const singular =
    (!data.inputs || data.inputs.length === 1) && (!data.outputs || data.outputs.length === 1);

  return (
    <>
      <div
        className={`flex relative w-[150px] z-[1001] rounded-sm bg-zinc-800 ${singular ? "h-[35px]" : undefined}`}
        style={{ zIndex: 1001 }}>
        <div className={`flex justify-center w-7 rounded-l-sm ${className}`}>
          {type === "reader" ? (
            <ReaderIcon className={typeIconClasses} />
          ) : type === "writer" ? (
            <DiscIcon className={typeIconClasses} />
          ) : type === "transformer" ? (
            <TransformerIcon className={typeIconClasses} />
          ) : null}
        </div>
        <p className="text-xs p-1.5 text-zinc-300 leading-none text-center self-center truncate">
          {data.name}
        </p>
      </div>
      {type !== "reader" && data.inputs && data.inputs.length === 1 && (
        <CustomHandle id="target" type="target" position={Position.Left} />
      )}
      {data.outputs && data.outputs.length === 1 && (
        <CustomHandle id="source" type="source" position={Position.Right} />
      )}
      <div
        id="handle-wrapper"
        className="absolute bg-zinc-700 rounded-b-md ml-auto mr-auto left-0 right-0 w-[90%]">
        {data.inputs &&
          data.inputs.length > 1 &&
          data.inputs.map((input, index) => (
            <div key={input + index} className="relative border-b border-zinc-900 py-0.5 px-1.5">
              <CustomHandle type="target" position={Position.Left} id={input} isConnectable={1} />
              <p className="text-xs pl-1">{input}</p>
            </div>
          ))}
        {data.outputs &&
          data.outputs.length > 1 &&
          data.outputs.map((output, index) => (
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
