import { Database, Disc, Graph, Lightning } from "@phosphor-icons/react";
import { NodeProps } from "@xyflow/react";
import { memo } from "react";

import type { Node } from "@flow/types";

import { Handles } from "./components";
import useHooks from "./hooks";

export type GeneralNodeProps = NodeProps<Node> & {
  className?: string;
};

const typeIconClasses = "w-[15px]";

const GeneralNode: React.FC<GeneralNodeProps> = ({
  className,
  data,
  type,
  selected,
}) => {
  const {
    officialName,
    inputs,
    outputs,
    // status,
    // intermediateDataUrl,
    borderColor,
    selectedColor,
    selectedBackgroundColor,
  } = useHooks({ data, type });

  return (
    <div
      className={`rounded-md bg-secondary border ${selected ? selectedColor : borderColor}`}>
      <div className="relative z-[1001] m-1 flex gap-1 h-[25px] min-w-[150px] max-w-[200px] rounded-sm">
        <div
          className={`flex p-1 self-center align-middle justify-center rounded-sm border ${selected ? selectedColor : borderColor} ${selected ? selectedBackgroundColor : className}`}>
          {type === "reader" ? (
            <Database className={typeIconClasses} />
          ) : type === "writer" ? (
            <Disc className={typeIconClasses} />
          ) : type === "transformer" ? (
            <Lightning className={typeIconClasses} />
          ) : type === "subworkflow" ? (
            <Graph className={typeIconClasses} />
          ) : null}
        </div>
        <div className="flex flex-1 items-center justify-between gap-2 truncate rounded-r-sm px-1 leading-none">
          <p className="self-center truncate text-xs text-gray-200">
            {data.customizations?.customName || officialName}
          </p>
        </div>
      </div>
      <Handles nodeType={type} inputs={inputs} outputs={outputs} />
    </div>
  );
};

export default memo(GeneralNode);
