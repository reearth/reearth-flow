import { ChevronDownIcon, ChevronUpIcon } from "@radix-ui/react-icons";
import { Position } from "@xyflow/react";
import { memo } from "react";

import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
  IconButton,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

import CustomHandle from "./CustomHandle";

type Props = {
  nodeType?: string;
  inputs?: string[];
  outputs?: string[];
  isCollapsed?: boolean;
  onCollapsedToggle?: (isCollapsed: boolean) => void;
};

const Handles: React.FC<Props> = ({
  nodeType,
  inputs,
  outputs,
  isCollapsed,
  onCollapsedToggle,
}) => {
  const t = useT();
  const hasMoreThanFiveOutputHandles = outputs && outputs.length >= 5;
  return (
    <Collapsible className="flex flex-col" open={!isCollapsed}>
      <div className="flex justify-between gap-0.5">
        {nodeType !== "reader" && inputs && (
          <div className="inset-x-0 mx-auto min-w-0 flex-1">
            {inputs.map((input, index) => (
              <div
                key={input + index}
                className="relative flex items-center border-b py-0.5 last-of-type:border-none">
                <CustomHandle
                  type="target"
                  className={`left-1 w-[8px] rounded-none transition-colors ${index === (!outputs && inputs && inputs.length - 1) ? "rounded-bl-sm" : undefined}`}
                  position={Position.Left}
                  id={input}
                  // isConnectable={1}
                />
                <div className="flex w-full translate-x-0.5 items-center">
                  <div>
                    <div className="size-1.5 rounded-full bg-gray-300" />
                  </div>
                  <p className="w-[90%] pl-1 text-[10px] break-words italic dark:font-thin">
                    {input}
                  </p>
                </div>
              </div>
            ))}
          </div>
        )}
        {outputs && !hasMoreThanFiveOutputHandles && (
          <div className="inset-x-0 mx-auto min-w-0 flex-1 overflow-hidden">
            {outputs.map((output, index) => (
              <div
                key={output + index}
                className="relative flex items-center justify-end border-b py-0.5 last-of-type:border-none">
                <CustomHandle
                  type="source"
                  className="right-1 z-10 w-[8px] rounded-none transition-colors"
                  position={Position.Right}
                  id={output}
                />
                <div className="flex w-full -translate-x-0.5 items-center justify-end">
                  <p className="w-[90%] pr-1 text-end text-[10px] break-words italic dark:font-thin">
                    {output}
                  </p>
                  <div className="size-1.5 rounded-full bg-gray-300" />
                </div>
              </div>
            ))}
          </div>
        )}
        {outputs && outputs.length >= 5 && isCollapsed && (
          <div className="inset-x-0 mx-auto min-w-0 flex-1 overflow-hidden">
            <div className="relative flex items-center justify-end border-b py-0.5 last-of-type:border-none">
              <div className="right-1 z-10 w-[8px] rounded-none transition-colors" />
              <div className="flex w-full -translate-x-0.5 items-center justify-end">
                <p className="w-[90%] pr-1 text-end text-[10px] break-words italic dark:font-thin">
                  {t("Multiple")}
                </p>
                <div className="size-1.5 rounded-full bg-gray-300" />
              </div>
            </div>
          </div>
        )}
      </div>
      {isCollapsed && (
        <>
          {outputs &&
            outputs.map((output, index) => (
              <CustomHandle
                key={`collapsed-${output}-${index}`}
                type="source"
                className="absolute right-1 z-10 w-[8px] rounded-none transition-colors"
                position={Position.Right}
                id={output}
                isConnectable={1}
                style={{ top: "55%", transform: "translateY(-50%)" }}
              />
            ))}
        </>
      )}
      <CollapsibleContent className="flex justify-between gap-2">
        <div className="inset-x-0 mx-auto min-w-0 flex-1" />
        {outputs && hasMoreThanFiveOutputHandles && (
          <div className="inset-x-0 mx-auto min-w-0 flex-1 overflow-hidden">
            {outputs.map((output, index) => (
              <div
                key={output + index}
                className="relative flex items-center justify-end  border-b py-0.5 last-of-type:border-none">
                <CustomHandle
                  type="source"
                  className="right-1 z-10 w-[8px] rounded-none transition-colors"
                  position={Position.Right}
                  id={output}
                />
                <div className="flex w-full -translate-x-0.5 items-center justify-end">
                  <p className="w-[90%] pr-1 text-end text-[10px] break-words italic dark:font-thin">
                    {output}
                  </p>
                  <div className="size-1.5 rounded-full bg-gray-300" />
                </div>
              </div>
            ))}
          </div>
        )}
      </CollapsibleContent>
      {outputs && outputs.length >= 5 && (
        <CollapsibleTrigger asChild className="justify-center self-center">
          <IconButton
            onClick={() => onCollapsedToggle?.(!isCollapsed)}
            className="h-6 w-6"
            icon={!isCollapsed ? <ChevronUpIcon /> : <ChevronDownIcon />}
          />
        </CollapsibleTrigger>
      )}
    </Collapsible>
  );
};

export default memo(Handles);
