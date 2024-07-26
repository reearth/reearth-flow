import { Database, Disc, Lightning } from "@phosphor-icons/react";
import { GearIcon, DoubleArrowRightIcon, PlayIcon } from "@radix-ui/react-icons";
import { NodeProps } from "@xyflow/react";
import { memo, useEffect, useState } from "react";

import { IconButton } from "@flow/components";
import { useDoubleClick } from "@flow/hooks";
import { Node } from "@flow/types";
import type { NodePosition, NodeType } from "@flow/types";

import { getPropsFrom } from "../utils";

import { Handles } from "./components";

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
      className="rounded-sm bg-secondary"
      // onMouseEnter={() => setHovered(true)}
      // onMouseLeave={() => setHovered(false)}
      onDoubleClick={handleDoubleClick}>
      <div className="relative z-[1001] flex h-[25px] w-[150px] rounded-sm">
        <div
          className={`flex w-4 justify-center rounded-l-sm border-y border-l ${selected ? (hardSelect ? "border-red-300" : "border-primary/50") : "border-primary/20"} ${className}`}>
          {type === "reader" ? (
            <Database className={typeIconClasses} />
          ) : type === "writer" ? (
            <Disc className={typeIconClasses} />
          ) : type === "transformer" ? (
            <Lightning className={typeIconClasses} />
          ) : null}
        </div>
        <div
          className={`flex flex-1 justify-between gap-2 truncate rounded-r-sm border-y border-r px-1 leading-none ${selected ? (hardSelect ? "border-red-300" : "border-primary/50") : "border-primary/20"}`}>
          <p className="self-center truncate text-[10px] font-light">{name}</p>
          <div className={`size-[8px] self-center rounded ${metaProps.style}`} />
        </div>
        {selected && !locked && (
          <div className="absolute bottom-[25px] right-1/2 flex h-[25px] w-[95%] translate-x-1/2 items-center justify-center rounded-t-lg bg-secondary">
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
