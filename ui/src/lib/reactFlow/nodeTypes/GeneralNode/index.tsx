import { Database, Disc, Eye, Graph, Lightning } from "@phosphor-icons/react";
import { NodeProps } from "@xyflow/react";
import { memo, useMemo } from "react";

import { Node } from "@flow/types";
import type { NodePosition, NodeType } from "@flow/types";
import { isDefined } from "@flow/utils";

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

const borderColorTypesObject = {
  reader: "border-node-reader",
  writer: "border-node-writer",
  transformer: "border-node-transformer",
  subworkflow: "border-node-subworkflow",
  default: "border-primary/20",
};

const selectedColorTypesObject = {
  reader: "border-node-reader-selected",
  writer: "border-node-writer-selected",
  transformer: "border-node-transformer-selected",
  subworkflow: "border-node-subworkflow-selected",
  default: "border-zinc-600",
};

const selectedColorTypesBackgroundsObject = {
  reader: "bg-node-reader-selected",
  writer: "bg-node-writer-selected",
  transformer: "bg-node-transformer-selected",
  subworkflow: "bg-node-subworkflow-selected",
  default: "bg-zinc-600",
};

const GeneralNode: React.FC<GeneralNodeProps> = ({
  className,
  data,
  type,
  selected,
}) => {
  const {
    officialName,
    customName,
    status,
    inputs: defaultInputs,
    outputs: defaultOutputs,
  } = data;

  const inputs: string[] = useMemo(() => {
    if (data.params?.conditions) {
      const i = data.params.conditions
        .map((condition: any) => condition.inputPort)
        .filter(isDefined);
      return i.length ? i : defaultInputs;
    }
    return defaultInputs;
  }, [data.params?.conditions, defaultInputs]);

  const outputs: string[] = useMemo(() => {
    if (data.params?.conditions) {
      const i = data.params.conditions
        .map((condition: any) => condition.outputPort)
        .filter(isDefined);
      return i.length ? i : defaultOutputs;
    }
    return defaultOutputs;
  }, [data.params?.conditions, defaultOutputs]);

  const metaProps = getPropsFrom(status);

  const borderColorTypes = Object.keys(borderColorTypesObject).includes(type)
    ? borderColorTypesObject[type as keyof typeof borderColorTypesObject]
    : borderColorTypesObject.default;

  const selectedColorTypes = Object.keys(selectedColorTypesObject).includes(
    type,
  )
    ? selectedColorTypesObject[type as keyof typeof selectedColorTypesObject]
    : selectedColorTypesObject.default;

  const selectedColorTypesBackgrounds = Object.keys(
    selectedColorTypesBackgroundsObject,
  ).includes(type)
    ? selectedColorTypesBackgroundsObject[
        type as keyof typeof selectedColorTypesBackgroundsObject
      ]
    : selectedColorTypesBackgroundsObject.default;

  return (
    <div className="rounded-sm bg-secondary">
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
          <p className="self-center truncate text-xs dark:font-light">
            {customName || officialName}
          </p>
          {status === "success" ? (
            <div className="self-center">
              <Eye />
            </div>
          ) : null}
          <div
            className={`size-[8px] shrink-0 self-center rounded ${metaProps.style}`}
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
