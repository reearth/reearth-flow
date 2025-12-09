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
const MIN_HANDLES_FOR_COLLAPSE = 5;
const Handles: React.FC<Props> = ({
  nodeType,
  inputs,
  outputs,
  isCollapsed,
  onCollapsedToggle,
}) => {
  const t = useT();
  const hasMoreThanFiveInputHandles =
    inputs && inputs.length >= MIN_HANDLES_FOR_COLLAPSE;
  const hasMoreThanFiveOutputHandles =
    outputs && outputs.length >= MIN_HANDLES_FOR_COLLAPSE;
  return (
    <Collapsible className="flex flex-col" open={!isCollapsed}>
      <div className="flex justify-between gap-0.5">
        {nodeType !== "reader" &&
          hasMoreThanFiveInputHandles &&
          isCollapsed && (
            <div className="inset-x-0 min-w-0 flex-1 overflow-hidden">
              <div className="relative flex items-center py-0.5">
                <div className="flex w-full translate-x-0.5 items-center">
                  <div className="flex items-center -space-x-0.75">
                    {Array.from({ length: 3 }).map((_, idx) => {
                      return (
                        <div
                          key={idx}
                          className="size-1.5 rounded-full bg-zinc-400 ring ring-secondary/20 dark:bg-gray-300"
                        />
                      );
                    })}
                  </div>

                  <p className="w-[90%] pl-1 text-[10px] break-words italic dark:font-thin">
                    {t("Multiple")}
                  </p>
                </div>
              </div>
            </div>
          )}
        {nodeType !== "reader" && inputs && !hasMoreThanFiveInputHandles && (
          <div className="inset-x-0 mx-auto min-w-0 flex-1">
            {inputs.map((input, index) => (
              <div
                key={input + index}
                className="relative flex items-center py-0.5">
                <CustomHandle
                  type="target"
                  className={`left-1 w-[8px] rounded-none transition-colors ${index === (!outputs && inputs && inputs.length - 1) ? "rounded-bl-sm" : undefined}`}
                  position={Position.Left}
                  id={input}
                />
                <div className="flex w-full translate-x-0.5 items-center">
                  <div>
                    <div className="size-1.5 rounded-full bg-zinc-400 dark:bg-gray-300" />
                  </div>
                  <p className="w-[90%] pl-1 text-[10px] break-words italic dark:font-thin">
                    {input}
                  </p>
                </div>
              </div>
            ))}
          </div>
        )}

        {hasMoreThanFiveOutputHandles && isCollapsed && (
          <div className="inset-x-0 mx-auto min-w-0 flex-1 overflow-hidden">
            <div className="relative flex items-center justify-end py-0.5">
              <div className="flex w-full -translate-x-0.5 items-center justify-end">
                <p className="w-[90%] pr-1 text-end text-[10px] break-words italic dark:font-thin">
                  {t("Multiple")}
                </p>
                <div className="flex items-center -space-x-0.75">
                  {Array.from({ length: 3 }).map((_, idx) => {
                    return (
                      <div
                        key={idx}
                        className="size-1.5 rounded-full bg-zinc-400 ring ring-secondary/20 dark:bg-gray-300"
                      />
                    );
                  })}
                </div>
              </div>
            </div>
          </div>
        )}

        {(hasMoreThanFiveInputHandles || hasMoreThanFiveOutputHandles) && (
          <CollapsibleContent className="inset-x-0 mx-auto min-w-0 flex-1">
            <div className="flex justify-between gap-0.5">
              {nodeType !== "reader" &&
                inputs &&
                hasMoreThanFiveInputHandles && (
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
                        />
                        <div className="flex w-full translate-x-0.5 items-center">
                          <div>
                            <div className="size-1.5 rounded-full bg-zinc-400 dark:bg-gray-300" />
                          </div>
                          <p className="w-[90%] pl-1 text-[10px] break-words italic dark:font-thin">
                            {input}
                          </p>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
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
                      <div className="flex -translate-x-0.5 items-center justify-end">
                        <p className="w-[90%] pr-1 text-end text-[10px] break-words italic dark:font-thin">
                          {output}
                        </p>
                        <div className="size-1.5 rounded-full bg-zinc-400 dark:bg-gray-300" />
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </CollapsibleContent>
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
                  <div className="size-1.5 rounded-full bg-zinc-400 dark:bg-gray-300" />
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
      {isCollapsed && (
        <>
          {nodeType !== "reader" &&
            inputs &&
            inputs.map((input, index) => (
              <CustomHandle
                key={`collapsed-input-${input}-${index}`}
                type="target"
                className="absolute left-1 z-10 w-[8px] rounded-none transition-colors"
                position={Position.Left}
                id={input}
                isConnectable={1}
                style={{ top: "44px" }}
              />
            ))}
          {outputs &&
            outputs.map((output, index) => (
              <CustomHandle
                key={`collapsed-output-${output}-${index}`}
                type="source"
                className="absolute right-1 z-10 w-[8px] rounded-none transition-colors"
                position={Position.Right}
                id={output}
                isConnectable={1}
                style={{ top: "44px" }}
              />
            ))}
        </>
      )}

      {((nodeType !== "reader" && hasMoreThanFiveInputHandles) ||
        hasMoreThanFiveOutputHandles) && (
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
