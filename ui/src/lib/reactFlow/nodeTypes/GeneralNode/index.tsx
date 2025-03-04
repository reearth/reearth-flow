import {
  Database,
  Disc,
  Eye,
  GearFine,
  Graph,
  Lightning,
  TrashSimple,
} from "@phosphor-icons/react";
import { NodeProps } from "@xyflow/react";
import { memo } from "react";

import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { Node } from "@flow/types";

import { Handles } from "./components";
import useHooks from "./hooks";

export type GeneralNodeProps = NodeProps<Node> & {
  className?: string;
};

const typeIconClasses = "w-[10px] h-[100%]";

const GeneralNode: React.FC<GeneralNodeProps> = ({
  className,
  data,
  type,
  selected,
}) => {
  const t = useT();

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
    <ContextMenu>
      <ContextMenuTrigger>
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
                {customName || officialName}
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
      </ContextMenuTrigger>
      <ContextMenuContent>
        <ContextMenuItem className="justify-between gap-4 text-xs">
          {t("Node Settings")}
          <GearFine weight="light" />
        </ContextMenuItem>
        {/* <ContextMenuItem
          className="justify-between gap-4 text-xs"
          disabled={!selected}>
          {t("Subworkflow from Selection")}
          <Graph weight="light" />
        </ContextMenuItem> */}
        <ContextMenuItem className="justify-between gap-4 text-xs" disabled>
          {t("Preview Intermediate Data")}
          <Eye weight="light" />
        </ContextMenuItem>
        <ContextMenuItem className="justify-between gap-4 text-xs text-destructive">
          {t("Delete Node")}
          <TrashSimple weight="light" />
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  );
};

export default memo(GeneralNode);
