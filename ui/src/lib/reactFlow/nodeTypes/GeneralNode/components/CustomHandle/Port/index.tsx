import { TableIcon, XCircleIcon } from "@phosphor-icons/react";
import { Position } from "@xyflow/react";
import { memo } from "react";

import { IconButton } from "@flow/components";
import { NodeData } from "@flow/types";

import CustomHandle from "../CustomHandle";
import { getBreakClass } from "../utils";

import useHooks from "./hooks";

type Props = {
  nodeId: string;
  nodeData: NodeData;
  portName: string;
  readonly: boolean;
};

const Port: React.FC<Props> = ({ nodeId, nodeData, portName, readonly }) => {
  const { hasIntermediateData, isSelected, jobStatus, handleClick } = useHooks({
    nodeId,
    nodeData,
    portName,
    readonly,
  });

  const hasData = hasIntermediateData && jobStatus === "completed";

  return (
    <div className="relative flex items-center justify-end border-b py-0.5 pl-0.5 last-of-type:border-none">
      <CustomHandle
        id={portName}
        type="source"
        className="right-1 z-10 w-[8px] rounded-none transition-colors"
        position={Position.Right}
        isConnectable={hasIntermediateData ? 1 : 0}
      />
      <div className="group flex w-full min-w-0 -translate-x-0.5 items-center justify-end gap-1">
        <div
          className={`flex items-center gap-1 rounded-sm  px-0.5 group-hover:cursor-pointer group-hover:bg-success/20 ${isSelected ? "bg-success/20" : ""}`}
          onClick={handleClick}>
          <p
            className={`min-w-0 text-end text-[10px] ${getBreakClass(portName)} italic dark:font-thin ${
              hasData && isSelected
                ? "text-success"
                : hasData
                  ? "text-success/60"
                  : ""
            }`}>
            {portName}
          </p>
          {hasData ? (
            <IconButton
              className={`z-11 h-3 w-3 shrink-0 text-success/60 hover:bg-transparent hover:text-success/60 ${isSelected ? "text-success hover:text-success" : ""}`}
              aria-label={`View intermediate data for ${portName}`}
              icon={<TableIcon />}
            />
          ) : jobStatus === "failed" ? (
            <XCircleIcon className="hover:text-error/60 z-11 h-3 w-3 shrink-0 text-destructive" />
          ) : (
            <div className="size-1.5 shrink-0 rounded-full bg-zinc-400 dark:bg-gray-300" />
          )}
        </div>
      </div>
    </div>
  );
};

export default memo(Port);
