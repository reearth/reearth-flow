import {
  DatabaseIcon,
  DiscIcon,
  GraphIcon,
  LightningIcon,
} from "@phosphor-icons/react";
import { NodeProps } from "@xyflow/react";
import { memo } from "react";

import type { Node } from "@flow/types";

import { Handles } from "./components";
import useHooks from "./hooks";

export type GeneralNodeProps = NodeProps<Node> & {
  className?: string;
};

const typeIconClasses = "w-[15px]";

const GeneralNode: React.FC<GeneralNodeProps> = ({
  data,
  type,
  selected,
  id,
}) => {
  const {
    officialName,
    inputs,
    outputs,
    // status,
    // intermediateDataUrl,
    backgroundColor,
    borderColor,
    selectedColor,
    selectedBackgroundColor,
    handleCollapsedToggle,
  } = useHooks({ data, type, nodeId: id });

  return (
    <div
      className={`max-w-[200px] min-w-[150px] rounded-lg border bg-secondary p-1 shadow-md shadow-secondary backdrop-blur-xs  ${selected ? selectedColor : borderColor} ${data.isDisabled ? "opacity-70" : ""}`}>
      <div className="relative flex h-[25px] items-center gap-1 rounded-sm">
        <div
          className={`flex justify-center self-center rounded-lg border p-1 align-middle ${selected ? selectedColor : borderColor} ${selected ? selectedBackgroundColor : backgroundColor}`}>
          {type === "reader" ? (
            <DatabaseIcon className={typeIconClasses} />
          ) : type === "writer" ? (
            <DiscIcon className={typeIconClasses} />
          ) : type === "transformer" ? (
            <LightningIcon className={typeIconClasses} />
          ) : type === "subworkflow" ? (
            <GraphIcon className={typeIconClasses} />
          ) : null}
        </div>
        <div className="flex flex-1 items-center justify-between gap-2 truncate rounded-r-sm px-1 leading-none">
          <p className="self-center truncate text-xs text-gray-200">
            {data.customizations?.customName || officialName}
          </p>
        </div>
        {/* <CaretRight weight="fill" /> */}
      </div>
      <Handles
        nodeType={type}
        inputs={inputs}
        outputs={outputs}
        isCollapsed={data.isCollapsed}
        onCollapsedToggle={handleCollapsedToggle}
      />
    </div>
  );
};

export default memo(GeneralNode);
