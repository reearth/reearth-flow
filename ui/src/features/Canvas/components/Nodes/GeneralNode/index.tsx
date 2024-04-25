import { DiscIcon, GearIcon, DoubleArrowRightIcon, PlayIcon } from "@radix-ui/react-icons";
import { useState } from "react";
import { NodeProps } from "reactflow";

import { IconButton, ReaderIcon, TransformerIcon } from "@flow/components";
import { useDoubleClick } from "@flow/hooks";
import { NodeData } from "@flow/types";

import { getPropsFrom } from "../utils";

import { Handles } from "./components/CustomHandle/Handles";
import type { NodePosition, NodeType } from "./types";

export type GeneralNodeProps = NodeProps<NodeData> & {
  className?: string;
  onHover?: (nodeInfo?: { id: string; type: NodeType; position: NodePosition }) => void;
};

const typeIconClasses = "w-[10px] h-[100%]";

const GeneralNode: React.FC<GeneralNodeProps> = ({ className, data, type, selected, ...props }) => {
  const [hovered, setHovered] = useState(false);

  const [_, handleDoubleClick] = useDoubleClick(undefined, () => console.log("double click"));
  // console.log("D", data);
  // const onChange = useCallback(
  //   (evt: any) => {
  //     console.log("EVT", evt.target.value);
  //     console.log("data", data);
  //   },
  //   [data],
  // );
  console.log("props: ", props);
  // console.log("data: ", data);

  const singular =
    (!data.inputs || data.inputs.length === 1) && (!data.outputs || data.outputs.length === 1);

  const metaProps = getPropsFrom(data.status);

  return (
    <div
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onDoubleClick={handleDoubleClick}>
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
      <Handles
        nodeType={type}
        inputs={data.inputs}
        outputs={data.outputs}
        nodeActionArea={
          hovered && (
            <div className="absolute flex items-center bg-zinc-800 rounded-b border-t border-zinc-700 right-[50%] translate-x-1/2">
              <IconButton size="icon" icon={<DoubleArrowRightIcon className="" />} />
              <IconButton size="icon" icon={<PlayIcon className="" />} />
              <IconButton size="icon" icon={<GearIcon className="" />} />
            </div>
          )
        }
      />
    </div>
  );
};

export default GeneralNode;
