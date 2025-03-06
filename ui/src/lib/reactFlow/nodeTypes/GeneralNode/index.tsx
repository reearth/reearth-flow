import { Database, Disc, Eye, Graph, Lightning } from "@phosphor-icons/react";
import { RJSFSchema } from "@rjsf/utils";
import { NodeProps } from "@xyflow/react";
import { memo } from "react";

import { Node } from "@flow/types";

import NodeContextMenu from "../NodeContextMenu";

import { Handles } from "./components";
import useHooks from "./hooks";

export type GeneralNodeProps = NodeProps<Node> & {
  className?: string;
};

const typeIconClasses = "w-[10px] h-[100%]";

export const generalNodeSchema = (officialName: string): RJSFSchema => ({
  type: "object",
  properties: {
    customName: {
      type: "string",
      title: "Name",
      format: "text",
      default: officialName,
    },
  },
});

const GeneralNode: React.FC<GeneralNodeProps> = ({
  className,
  id,
  data,
  type,
  selected,
}) => {
  const {
    officialName,
    customName,
    inputs,
    outputs,
    status,
    metaProps,
    borderColor,
    selectedColor,
    selectedBackgroundColor,
  } = useHooks({ data, type });

  return (
    <NodeContextMenu nodeId={id} nodeType={type}>
      <div className="rounded-sm bg-secondary">
        <div className="relative z-[1001] flex h-[25px] w-[150px] rounded-sm">
          <div
            className={`flex w-4 justify-center rounded-l-sm border-y border-l ${status === "failed" ? "border-destructive" : selected ? selectedColor : borderColor} ${selected ? selectedBackgroundColor : className} `}>
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
            className={`flex flex-1 justify-between gap-2 truncate rounded-r-sm border-y border-r px-1 leading-none ${status === "failed" ? "border-destructive" : selected ? selectedColor : borderColor}`}>
            <p className="self-center truncate text-xs dark:font-light">
              {data.customizations?.customName || customName || officialName}
            </p>
            {status === "succeeded" ? (
              <div className="self-center">
                <Eye />
              </div>
            ) : null}
            <div
              className={`size-[8px] shrink-0 self-center rounded ${metaProps.style}`}
            />
          </div>
        </div>
        <Handles nodeType={type} inputs={inputs} outputs={outputs} />
      </div>
    </NodeContextMenu>
  );
};

export default memo(GeneralNode);
