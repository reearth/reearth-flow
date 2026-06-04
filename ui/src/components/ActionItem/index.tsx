import { forwardRef } from "react";

import { cn } from "@flow/lib/utils";
import { Action } from "@flow/types";
import { typeColorClass } from "@flow/utils";
import { getNodeIcon } from "@flow/utils/getNodeIcon";

type Props = {
  itemRefs: React.RefObject<(HTMLDivElement | null)[]>;
  idx: number;
  action: Action;
  isSelected?: boolean;
  onSingleClick?: (actionName: string) => void;
  onDoubleClick?: (actionName: string) => void;
};

const ActionItem = forwardRef<HTMLDivElement, Props>(
  ({ itemRefs, idx, action, isSelected, onSingleClick, onDoubleClick }) => {
    const Icon = getNodeIcon(action.type);

    return (
      <>
        <div
          ref={(el) => {
            itemRefs.current[idx] = el;
          }}
          className={cn(
            "flex cursor-pointer items-center gap-2 rounded-xl border border-primary bg-secondary px-2 py-1.5",
            isSelected
              ? "bg-primary/75 text-accent-foreground"
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
          <span className="flex-1 truncate text-sm select-none">
            {action.name}
          </span>
          <div className="self-center rounded border bg-secondary/80 px-1 py-0.5 align-middle">
            <p className="text-xs capitalize select-none">
              {action.categories[0] ?? action.type}
            </p>
          </div>
        </div>
        {/* {idx !== actionsList.length - 1 && <div className="mx-1 border-b" />} */}
      </>
    );
  },
);

export default ActionItem;
