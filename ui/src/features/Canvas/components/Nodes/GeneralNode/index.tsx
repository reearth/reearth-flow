import { DiscIcon } from "@radix-ui/react-icons";
import { NodeProps, Position } from "reactflow";

import { ReaderIcon, TransformerIcon } from "@flow/components";
import { NodeData } from "@flow/types";

import { getPropsFrom } from "../utils";

import CustomHandle from "./CustomHandle";
import type { NodePosition, NodeType } from "./types";

export type GeneralNodeProps = NodeProps<NodeData> & {
  className?: string;
  onHover?: (nodeInfo?: { id: string; type: NodeType; position: NodePosition }) => void;
};

const typeIconClasses = "w-[10px] h-[100%]";

const GeneralNode: React.FC<GeneralNodeProps> = ({ className, data, type, selected, ...props }) => {
  // console.log("D", data);
  // const onChange = useCallback(
  //   (evt: any) => {
  //     console.log("EVT", evt.target.value);
  //     console.log("data", data);
  //   },
  //   [data],
  // );
  console.log("props: ", props);
  console.log("data: ", data);

  const singular =
    (!data.inputs || data.inputs.length === 1) && (!data.outputs || data.outputs.length === 1);

  const metaProps = getPropsFrom(data.status);

  return (
    <>
      <div
        className={`flex relative w-[150px] z-[1001] rounded-sm bg-zinc-800 ${singular ? "h-[30px]" : "h-[25px]"}`}
        style={{ zIndex: 1001 }}>
        <div
          className={`flex justify-center w-4 rounded-l-sm border-t border-l border-b ${selected ? "border-zinc-400" : "border-zinc-700"} ${className}`}>
          {type === "reader" ? (
            <ReaderIcon className={typeIconClasses} />
          ) : type === "writer" ? (
            <DiscIcon className={typeIconClasses} />
          ) : type === "transformer" ? (
            <TransformerIcon className={typeIconClasses} />
          ) : null}
        </div>
        <div
          className={`flex justify-between gap-2 flex-1 px-1 leading-none truncate rounded-r-sm border-t border-r border-b ${selected ? "border-zinc-400" : "border-zinc-700"}`}>
          <p className="text-[10px] text-zinc-300 font-light truncate self-center">{data.name}</p>
          <div className={`w-[8px] h-[8px] rounded self-center ${metaProps.style}`} />
        </div>
      </div>
      {type !== "reader" && data.inputs && data.inputs.length === 1 && (
        <CustomHandle
          id="target"
          className="rounded-l rounded-r-none -left-0 z-[1001] w-[16px]"
          type="target"
          position={Position.Left}
        />
      )}
      {data.outputs && data.outputs.length === 1 && (
        <CustomHandle
          id="source"
          className="rounded-r rounded-l-none -right-0 z-[1001] w-[16px]"
          type="source"
          position={Position.Right}
        />
      )}
      <div
        id="handle-wrapper"
        className="absolute bg-zinc-800 text-zinc-400 rounded-b-md ml-auto mr-auto left-0 right-0 w-[95%]">
        {data.inputs &&
          data.inputs.length > 1 &&
          data.inputs.map((input, index) => (
            <div key={input + index} className="relative border-b border-zinc-900 py-0.5 px-1.5">
              <CustomHandle
                type="target"
                className={`left-0 w-[8px] rounded-none transition-colors ${index === (!data.outputs && data.inputs && data.inputs.length - 1) ? "rounded-bl-md" : undefined}`}
                position={Position.Left}
                id={input}
                // isConnectable={1}
              />
              <p className="text-[10px] font-light pl-1">{input}</p>
            </div>
          ))}
        {data.outputs &&
          data.outputs.length > 1 &&
          data.outputs.map((output, index) => (
            <div
              key={output + index}
              className="relative border-b border-zinc-900 py-0.5 px-1.5 last-of-type:border-none">
              <CustomHandle
                type="source"
                className={`right-0 w-[8px] rounded-none transition-colors ${index === (data.outputs && data.outputs.length - 1) ? "rounded-br-md" : undefined}`}
                position={Position.Right}
                id={output}
              />
              <p className="text-[10px] font-light pr-1 text-end">{output}</p>
            </div>
          ))}
      </div>
    </>
  );
};

export default GeneralNode;
