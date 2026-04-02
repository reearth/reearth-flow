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

  return (
    <div className="relative flex items-center justify-end border-b py-0.5 last-of-type:border-none">
      <CustomHandle
        id={portName}
        type="source"
        className="right-1 z-10 w-[8px] rounded-none transition-colors"
        position={Position.Right}
        isConnectable={hasIntermediateData ? 1 : 0}
      />
      <div className="flex w-full min-w-0 -translate-x-0.5 items-center justify-end gap-1">
        <p
          className={`min-w-0 text-end text-[10px] ${getBreakClass(portName)} italic dark:font-thin`}>
          {portName}
        </p>
        {hasIntermediateData && jobStatus === "completed" ? (
          <IconButton
            className={`z-11 h-3 w-3 shrink-0 hover:bg-transparent hover:text-success ${
              jobStatus === "completed" && isSelected
                ? "text-success"
                : "text-success/60"
            } `}
            aria-label={`View intermediate data for ${portName}`}
            icon={<TableIcon />}
            onClick={handleClick}
          />
        ) : jobStatus === "failed" ? (
          <XCircleIcon
            className={`hover:text-error/60 z-11 h-3 w-3 shrink-0 text-destructive`}
          />
        ) : (
          <div className="size-1.5 shrink-0 rounded-full bg-zinc-400 dark:bg-gray-300" />
        )}
      </div>
    </div>
  );
};

export default memo(Port);
