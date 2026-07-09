import { ChevronDownIcon, ChevronUpIcon } from "@radix-ui/react-icons";
import { Position, useUpdateNodeInternals } from "@xyflow/react";
import { memo, useEffect, useMemo, useRef } from "react";

import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
  IconButton,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { NodeData } from "@flow/types";

import SchemaIndicator from "../SchemaIndicator";

import CustomHandle from "./CustomHandle";
import Port from "./Port";
import { getBreakClass } from "./utils";

type Props = {
  id: string;
  readonly: boolean;
  nodeType?: string;
  nodeData: NodeData;
  inputs?: string[];
  outputs?: string[];
  isCollapsed?: boolean;
  onCollapsedToggle?: (isCollapsed: boolean) => void;
};

const MIN_HANDLES_FOR_COLLAPSE = 5;

const Handles: React.FC<Props> = ({
  id,
  readonly,
  nodeType,
  nodeData,
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

  /** React Flow caches handle positions and only re-measures when a node's size
   * changes — not when a handle's id changes in place (e.g. a subworkflow
   * pseudoport renamed by a collaborator). Without a nudge, edges to the
   * changed handle can't resolve and drop until the canvas remounts. Re-measure
   * whenever the handle set changes; skip the first render since mount is already measured.
   */

  const updateNodeInternals = useUpdateNodeInternals();
  const handleSignature = useMemo(
    () =>
      `${inputs?.join(",") ?? ""}|${outputs?.join(",") ?? ""}|${nodeData.isCollapsed ? 1 : 0}`,
    [inputs, outputs, nodeData.isCollapsed],
  );

  const isFirstRender = useRef(true);
  useEffect(() => {
    if (isFirstRender.current) {
      isFirstRender.current = false;
      return;
    }
    updateNodeInternals(id);
  }, [id, handleSignature, updateNodeInternals]);

  return (
    <Collapsible className="flex flex-col" open={!isCollapsed}>
      <div className="flex justify-between gap-0.5">
        {nodeType === "reader" && (
          <SchemaIndicator schema={nodeData.nodeMetadata?.schema} />
        )}
        {nodeType !== "reader" &&
          hasMoreThanFiveInputHandles &&
          isCollapsed && (
            <div className="inset-x-0 min-w-0 flex-1 overflow-hidden">
              <div className="relative flex items-center py-0.5">
                <div className="flex w-full translate-x-0.5 items-center">
                  <div className="flex items-center -space-x-0.75">
                    {Array.from({ length: 3 }).map((_, idx) => (
                      <div
                        key={idx}
                        className="size-1.5 rounded-full bg-zinc-400 ring ring-secondary/20 dark:bg-gray-300"
                      />
                    ))}
                  </div>
                  <p className="w-[90%] pl-1 text-[10px] wrap-break-word italic dark:font-thin">
                    {t("Multiple")}
                  </p>
                </div>
              </div>
            </div>
          )}

        {nodeType !== "reader" && inputs && !hasMoreThanFiveInputHandles && (
          <div className="inset-x-0 mx-auto min-w-0 flex-1 ">
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
                  <p
                    className={`w-[90%] pl-1 text-[10px] ${getBreakClass(input)} italic dark:font-thin`}>
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
                <p className="w-[90%] pr-1 text-end text-[10px] wrap-break-word italic dark:font-thin">
                  {t("Multiple")}
                </p>
                <div className="flex items-center -space-x-0.75">
                  {Array.from({ length: 3 }).map((_, idx) => (
                    <div
                      key={idx}
                      className="size-1.5 rounded-full bg-zinc-400 ring ring-secondary/20 dark:bg-gray-300"
                    />
                  ))}
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
                          <p
                            className={`w-[90%] pl-1 text-[10px] ${getBreakClass(input)} italic dark:font-thin`}>
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
                    <Port
                      key={output + index}
                      nodeId={id}
                      nodeData={nodeData}
                      portName={output}
                      readonly={readonly}
                    />
                  ))}
                </div>
              )}
            </div>
          </CollapsibleContent>
        )}
        {outputs && !hasMoreThanFiveOutputHandles && (
          <div className="inset-x-0 mx-auto min-w-0 flex-1 overflow-hidden">
            {outputs.map((output, index) => (
              <Port
                key={output + index}
                nodeId={id}
                nodeData={nodeData}
                portName={output}
                readonly={readonly}
              />
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
        <CollapsibleTrigger
          className="justify-center self-center"
          render={
            <IconButton
              onClick={() => onCollapsedToggle?.(!isCollapsed)}
              className="h-6 w-6"
              icon={!isCollapsed ? <ChevronUpIcon /> : <ChevronDownIcon />}
            />
          }
        />
      )}
    </Collapsible>
  );
};

export default memo(Handles);
