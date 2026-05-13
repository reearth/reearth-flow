import { forwardRef } from "react";

import { typeColorClass } from "@flow/features/Editor/components/OverlayUI/components/ActionPickerDialog/utils";
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
          <div
            className={cn(
              "self-center rounded border p-1 align-middle",
              typeColorClass(action.type),
            )}>
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
