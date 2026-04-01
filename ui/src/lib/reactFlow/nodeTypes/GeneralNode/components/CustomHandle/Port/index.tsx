import { EyeIcon } from "@phosphor-icons/react";
import { Position } from "@xyflow/react";
import { memo } from "react";

import { IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { NodeData } from "@flow/types";

import CustomHandle from "../CustomHandle";

import useHooks from "./hooks";

const getBreakClass = (text: string): string =>
  /\s/.test(text) ? "break-words" : "break-all";

type Props = {
  nodeId: string;
  nodeData: NodeData;
  portName: string;
  nodeType?: string;
};

const Port: React.FC<Props> = ({ nodeId, nodeData, portName, nodeType }) => {
  const { hasIntermediateData, isSelected, jobStatus, handleClick } = useHooks({
    nodeId,
    nodeData,
    portName,
    nodeType,
  });
  const t = useT();
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
          {/* {hasIntermediateData && (
            <IconButton
              className={`h-4 w-4 ${
                jobStatus === "completed" && isSelected
                  ? "text-success"
                  : "text-success/60"
              }`}
              icon={<EyeIcon />}
              onClick={handleClick}
            />
          )} */}
          <p
            className={`pr-1 text-end text-[10px] ${getBreakClass(portName)} italic dark:font-thin`}>
            {portName}
          </p>
        </div>
        {hasIntermediateData ? (
          <IconButton
            className={`h-4 w-4 ${
              jobStatus === "completed" && isSelected
                ? "text-success"
                : "text-success/60"
            }`}
            tooltipText={t("View Intermediate Data")}
            icon={<EyeIcon />}
            onClick={handleClick}
          />
        ) : (
          <div className="size-1.5 rounded-full bg-zinc-400 dark:bg-gray-300" />
        )}

        {/* {hasIntermediateData ? (
          <div
            onClick={handleClick}
            className={`size-1.5 rounded-full ${
              jobStatus === "completed" && isSelected
                ? "bg-success"
                : "bg-success/60"
            }`}
          />
        ) : (
          <div className="size-1.5 rounded-full bg-zinc-400 dark:bg-gray-300" />
        )} */}
      </div>
    </div>
  );
};

export default memo(Port);
