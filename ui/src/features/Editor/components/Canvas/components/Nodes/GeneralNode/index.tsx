import { Database, Disc, Eye, Graph, Lightning } from "@phosphor-icons/react";
import { NodeProps } from "@xyflow/react";
import { memo } from "react";

import { Node } from "@flow/types";
import type { NodePosition, NodeType } from "@flow/types";

import { getPropsFrom } from "../utils";

import { Handles } from "./components";

export type GeneralNodeProps = NodeProps<Node> & {
  className?: string;
  onHover?: (nodeInfo?: {
    id: string;
    type: NodeType;
    position: NodePosition;
  }) => void;
};

const typeIconClasses = "w-[10px] h-[100%]";

const GeneralNode: React.FC<GeneralNodeProps> = ({
  className,
  data,
  type,
  selected,
}) => {
  const { officialName, customName, status, inputs, outputs } = data;

  const metaProps = getPropsFrom(status);

  const borderColorTypes =
    type === "reader"
      ? "border-node-reader/60"
      : type === "writer"
        ? "border-node-writer/60"
        : type === "transformer"
          ? "border-node-transformer/60"
          : type === "subworkflow"
            ? "border-node-subworkflow/60"
            : "border-primary/20";

  const selectedColorTypes =
    type === "reader"
      ? "border-node-reader-selected"
      : type === "writer"
        ? "border-node-writer-selected"
        : type === "transformer"
          ? "border-node-transformer-selected"
          : type === "subworkflow"
            ? "border-node-subworkflow-selected"
            : "border-zinc-600";

  const selectedColorTypesBackgrounds =
    type === "reader"
      ? "bg-node-reader-selected"
      : type === "writer"
        ? "bg-node-writer-selected"
        : type === "transformer"
          ? "bg-node-transformer-selected"
          : type === "subworkflow"
            ? "bg-node-subworkflow-selected"
            : "bg-zinc-600";

  return (
    <div className="rounded-sm  bg-secondary">
      <div className="relative z-[1001] flex h-[25px] w-[150px] rounded-sm">
        <div
          className={`flex w-4 justify-center rounded-l-sm border-y border-l ${selected ? selectedColorTypes : borderColorTypes} ${selected ? selectedColorTypesBackgrounds : className} `}>
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
          className={`flex flex-1 justify-between gap-2 truncate rounded-r-sm border-y border-r px-1 leading-none ${selected ? selectedColorTypes : borderColorTypes}`}>
          <p className="self-center truncate text-[10px] dark:font-light">
            {customName || officialName}
          </p>
          {status === "success" ? (
            <div className="self-center">
              <Eye />
            </div>
          ) : null}
          <div
            className={`size-[8px] self-center rounded ${metaProps.style}`}
          />
        </div>
        {/* {selected && (
          <div className="absolute bottom-[25px] right-1/2 flex h-[25px] w-[95%] translate-x-1/2 items-center justify-center rounded-t-lg bg-secondary">
            <IconButton
              className="h-full flex-1 rounded-b-none"
              size="icon"
              icon={<DoubleArrowRightIcon />}
            />
            <IconButton
              className="h-full flex-1 rounded-b-none"
              size="icon"
              icon={<PlayIcon />}
            />
            <IconButton
              className="h-full flex-1 rounded-b-none"
              size="icon"
              icon={<GearIcon />}
            />
          </div>
        )} */}
      </div>
      <Handles nodeType={type} inputs={inputs} outputs={outputs} />
    </div>
  );
};

export default memo(GeneralNode);
