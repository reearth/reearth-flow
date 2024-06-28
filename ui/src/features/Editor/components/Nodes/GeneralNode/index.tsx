import { Database, Disc, Lightning } from "@phosphor-icons/react";
import { GearIcon, DoubleArrowRightIcon, PlayIcon } from "@radix-ui/react-icons";
import { NodeProps } from "@xyflow/react";
import { memo, useEffect, useState } from "react";

import { IconButton } from "@flow/components";
import { useDoubleClick } from "@flow/hooks";
import { Node } from "@flow/types";

import { getPropsFrom } from "../utils";

import { Handles } from "./components";
import type { NodePosition, NodeType } from "./types";

export type GeneralNodeProps = NodeProps<Node> & {
  className?: string;
  onHover?: (nodeInfo?: { id: string; type: NodeType; position: NodePosition }) => void;
};

const typeIconClasses = "w-[10px] h-[100%]";

const GeneralNode: React.FC<GeneralNodeProps> = ({ className, data, type, selected, id }) => {
  // const [hovered, setHovered] = useState(false);

  const { name, status, inputs, outputs, locked, onLock } = data;

  const [hardSelect, setHardSelect] = useState<boolean>(!!locked);

  const [_, handleDoubleClick] = useDoubleClick(undefined, () => {
    setHardSelect(!hardSelect);
    onLock?.(id);
  });

  useEffect(() => {
    if (!selected && hardSelect) {
      setHardSelect(false);
      onLock?.(id);
    }
  }, [id, selected, hardSelect, onLock]);
  // console.log("D", data);
  // const onChange = useCallback(
  //   (evt: any) => {
  //     console.log("EVT", evt.target.value);
  //     console.log("data", data);
  //   },
  //   [data],
  // );
  // console.log("props: ", props);
  // console.log("data: ", data);

  const metaProps = getPropsFrom(status);

  return (
    <div
      className="bg-zinc-800 rounded-sm"
      // onMouseEnter={() => setHovered(true)}
      // onMouseLeave={() => setHovered(false)}
      onDoubleClick={handleDoubleClick}>
      <div
        className="flex relative w-[150px] z-[1001] rounded-sm bg-zinc-900/50 h-[25px]"
        style={{ zIndex: 1001 }}>
        <div
          className={`flex justify-center w-4 rounded-l-sm border-t border-l border-b ${selected ? (hardSelect ? "border-red-300" : "border-zinc-400") : "border-zinc-500"} ${className}`}>
          {type === "reader" ? (
            <Database className={typeIconClasses} />
          ) : type === "writer" ? (
            <Disc className={typeIconClasses} />
          ) : type === "transformer" ? (
            <Lightning className={typeIconClasses} />
          ) : null}
        </div>
        <div
          className={`flex justify-between gap-2 flex-1 px-1 leading-none truncate rounded-r-sm border-t border-r border-b ${selected ? (hardSelect ? "border-red-300" : "border-zinc-400") : "border-zinc-500"}`}>
          <p className="text-[10px] text-zinc-300 font-light truncate self-center">{name}</p>
          <div className={`w-[8px] h-[8px] rounded self-center ${metaProps.style}`} />
        </div>
        {selected && !locked && (
          <div className="absolute flex items-center justify-center h-[25px] w-[95%] bg-zinc-900 rounded-t-lg bottom-[25px] right-[50%] translate-x-1/2">
            <IconButton
              className="h-full flex-1 rounded-b-none"
              size="icon"
              icon={<DoubleArrowRightIcon />}
            />
            <IconButton className="h-full flex-1 rounded-b-none" size="icon" icon={<PlayIcon />} />
            <IconButton className="h-full flex-1 rounded-b-none" size="icon" icon={<GearIcon />} />
          </div>
        )}
      </div>
      <Handles nodeType={type} inputs={inputs} outputs={outputs} />
    </div>
  );
};

export default memo(GeneralNode);
