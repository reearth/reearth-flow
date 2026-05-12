import { forwardRef } from "react";

import { cn } from "@flow/lib/utils";
import { Action } from "@flow/types";
import { getNodeIcon } from "@flow/utils/getNodeIcon";

type Props = {
  itemRefs: React.RefObject<(HTMLDivElement | null)[]>;
  actionsList: Action[];
  idx: number;
  action: Action;
  isSelected?: boolean;
  onSingleClick?: (actionName: string) => void;
  onDoubleClick?: (actionName: string) => void;
};

const typeColorClass = (type: string) => {
  switch (type) {
    case "transformer":
      return "bg-node-transformer/80";
    case "reader":
      return "bg-node-reader/80";
    case "writer":
      return "bg-node-writer/80";
    default:
      return "bg-secondary";
  }
};

const ActionItem = forwardRef<HTMLDivElement, Props>(
  ({
    itemRefs,
    idx,
    action,
    isSelected,
    actionsList,
    onSingleClick,
    onDoubleClick,
  }) => {
    const Icon = getNodeIcon(action.type);

    return (
      <>
        <div
          ref={(el) => {
            itemRefs.current[idx] = el;
          }}
          className={cn(
            "flex cursor-pointer items-center gap-2 rounded px-2 py-1.5",
            isSelected
              ? "bg-primary text-accent-foreground"
              : "hover:bg-primary hover:text-accent-foreground",
          )}
          onClick={() => onSingleClick?.(action.name)}
          onDoubleClick={() => onDoubleClick?.(action.name)}>
          <div
            className={cn(
              "shrink-0 rounded p-0.5",
              typeColorClass(action.type),
            )}>
            <Icon size={12} weight="thin" className="text-white" />
          </div>
          <span className="flex-1 truncate text-sm">{action.name}</span>
          {/* <span className="shrink-0 text-xs capitalize opacity-60">
                              {action.type}
                            </span> */}
          <div
            className={`self-center rounded border  ${action.type === "transformer" ? "bg-node-transformer/95 dark:bg-node-transformer/60" : action.type === "reader" ? "bg-node-reader/95 dark:bg-node-reader/60" : action.type === "writer" ? "bg-node-writer/85 dark:bg-node-writer/30" : "bg-popover"} p-0.5 align-middle`}>
            <p className="self-center text-xs text-zinc-200 capitalize">
              {action.type}
            </p>
          </div>
        </div>
        {idx !== actionsList.length - 1 && <div className="mx-1 border-b" />}
      </>
    );
  },
);

export default ActionItem;
