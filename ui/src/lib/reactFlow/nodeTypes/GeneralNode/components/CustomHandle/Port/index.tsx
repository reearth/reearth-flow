import { EyeIcon } from "@phosphor-icons/react";
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
        type="source"
        className="right-1 z-10 w-[8px] rounded-none transition-colors"
        position={Position.Right}
        id={portName}
      />
      <div className="flex w-full -translate-x-0.5 items-center justify-end">
        <div className="flex items-center gap-1">
          <p
            className={`pr-1 text-end text-[10px] ${getBreakClass(portName)} italic dark:font-thin`}>
            {portName}
          </p>
        </div>
      </div>
      {hasIntermediateData ? (
        <IconButton
          className={`z-10 h-3 w-3 hover:text-success ${
            jobStatus === "completed" && isSelected
              ? "text-success"
              : "text-success/60"
          } `}
          icon={<EyeIcon />}
          onClick={handleClick}
        />
      ) : (
        <div className="size-1.5 rounded-full bg-zinc-400 dark:bg-gray-300" />
      )}
    </div>
  );
};

export default memo(Port);
