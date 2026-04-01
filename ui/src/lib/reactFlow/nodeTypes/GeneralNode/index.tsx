import {
  DatabaseIcon,
  DiscIcon,
  GraphIcon,
  LightningIcon,
} from "@phosphor-icons/react";
import { NodeProps } from "@xyflow/react";
import { memo, useMemo } from "react";

import { useAwarenessNodeSelections } from "@flow/features/Editor/editorContext";
import type { Node } from "@flow/types";

import { Handles } from "./components";
import useHooks from "./hooks";

export type GeneralNodeProps = NodeProps<Node> & {
  className?: string;
};

const typeIconClasses = "w-[15px] text-white/80";

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

  const awarenessSelections = useAwarenessNodeSelections(id);
  const remoteColor = awarenessSelections[0]?.color;

  const gradientBorderStyle = useMemo(() => {
    if (awarenessSelections.length < 2) return undefined;
    const colors = awarenessSelections.map((s) => s.color).join(", ");
    return {
      border: "1px solid transparent",
      background: `linear-gradient(var(--secondary), var(--secondary)) padding-box, linear-gradient(135deg, ${colors}) border-box`,
    };
  }, [awarenessSelections]);

  return (
    <div
      style={gradientBorderStyle}
      className={`max-w-[200px] min-w-[150px] rounded-lg bg-secondary shadow-md shadow-[black]/10 backdrop-blur-xs dark:shadow-secondary ${gradientBorderStyle ? "" : `border ${selected ? selectedColor : borderColor}`} ${data.isDisabled ? "opacity-70" : ""}`}>
      <div
        style={
          !gradientBorderStyle && remoteColor
            ? { outline: `solid ${remoteColor}` }
            : undefined
        }
        className={`rounded-[6px] border p-1 ${
          selected ? selectedColor : "border-transparent"
        }`}>
        <div className="relative flex h-[25px] items-center gap-1 rounded-sm">
          <div
            className={`flex justify-center self-center rounded-lg border p-1 align-middle ${selected ? `${selectedColor} ${selectedBackgroundColor} border-opacity-100` : `${borderColor} ${backgroundColor} border-opacity-0`}`}>
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
            <p className="self-center truncate text-xs dark:text-gray-200">
              {data.customizations?.customName || officialName}
            </p>
          </div>
          {/* <CaretRight weight="fill" /> */}
        </div>
        <Handles
          id={id}
          nodeType={type}
          nodeData={data}
          inputs={inputs}
          outputs={outputs}
          isCollapsed={data.isCollapsed}
          onCollapsedToggle={handleCollapsedToggle}
        />
      </div>
    </div>
  );
};

export default memo(GeneralNode);
