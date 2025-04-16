import { Database, Disc, Graph, Lightning } from "@phosphor-icons/react";
import { NodeProps } from "@xyflow/react";
import { memo } from "react";

import type { Node } from "@flow/types";

import { Handles } from "./components";
import useHooks from "./hooks";

export type GeneralNodeProps = NodeProps<Node> & {
  className?: string;
};

const typeIconClasses = "w-[10px] h-[100%]";

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
    <div className="rounded-sm bg-secondary">
      {/* <div
          className={`rounded-sm bg-secondary ${status === "processing" ? "active-node-status-shadow" : status === "pending" ? "queued-node-status-shadow" : ""}`}> */}
      <div className="relative z-[1001] flex h-[25px] w-[150px] rounded-sm">
        <div
          className={`flex w-4 justify-center rounded-l-sm border-y border-l ${selected ? selectedColor : borderColor} ${selected ? selectedBackgroundColor : className}`}>
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
        <div
          className={`flex flex-1 items-center justify-between gap-2 truncate rounded-r-sm border-y border-r px-1 leading-none ${selected ? selectedColor : borderColor}`}>
          <p className="self-center truncate text-xs dark:font-light">
            {data.customizations?.customName || officialName}
          </p>
          {/* {status === "failed" && <X className="size-4 text-destructive" />} */}
          {/* {status === "completed" && intermediateDataUrl && (
                <Table className="size-4 text-success" />
              )} */}
        </div>
      </div>
      <Handles nodeType={type} inputs={inputs} outputs={outputs} />
    </div>
  );
};

export default memo(GeneralNode);
