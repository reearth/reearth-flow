import { DragEvent, MouseEvent, useCallback } from "react";

import { Action } from "@flow/types";

type Props = {
  className?: string;
  action: Action;
  selected: boolean | undefined;
  draggable?: boolean;
  onMouseDown?: () => void;
  onTypeClick?: (type: string) => void;
  onCategoryClick?: (category: string) => void;
  onDragStart?: (event: DragEvent<HTMLDivElement>, actionName: string) => void;
  onSingleClick?: (name?: string) => void;
  onDoubleClick?: (name?: string) => void;
};

const ActionItem: React.FC<Props> = ({
  className,
  action,
  selected,
  draggable,
  onMouseDown,
  onTypeClick,
  onCategoryClick,
  onDragStart,
  onSingleClick,
  onDoubleClick,
}) => {
  const handleTypeClick = useCallback(
    (type: string) => (e: MouseEvent) => {
      if (!onTypeClick) return;
      e.stopPropagation();
      onTypeClick(type);
    },
    [onTypeClick],
  );

  const handleCategoryClick = useCallback(
    (category: string) => (e: MouseEvent) => {
      e.stopPropagation();
      onCategoryClick?.(category);
    },
    [onCategoryClick],
  );
  return (
    <div
      key={action.name}
      className={`group cursor-pointer rounded p-2 ${selected ? "bg-primary text-accent-foreground" : "hover:bg-primary hover:text-accent-foreground"} ${className}`}
      onClick={() => onSingleClick?.(action.name)}
      onDoubleClick={() => onDoubleClick?.(action.name)}
      draggable={draggable}
      onMouseDown={onMouseDown}
      onDragStart={(e) => onDragStart?.(e, action.name)}>
      <div className="flex w-full justify-between gap-1 pb-2">
        <div className="w-3/5 self-center break-words text-sm">
          <p className="self-center text-zinc-200">{action.name}</p>
        </div>
        <div
          className={`self-center rounded border ${action.type === "transformer" ? "bg-node-transformer/30" : action.type === "reader" ? "bg-node-reader/30" : action.type === "writer" ? "bg-node-writer/30" : "bg-popover"} p-1 align-middle`}
          onClick={handleTypeClick(action.type)}>
          <p className="self-center text-xs capitalize text-zinc-200">
            {action.type}
          </p>
        </div>
      </div>
      <div className="group-hover:block">
        <div className="mb-2 text-xs leading-[0.85rem]">
          {action.description}
        </div>
        <div className="flex flex-wrap gap-1 text-xs">
          {action.categories.map((c) => (
            <div
              className="rounded border bg-popover p-[2px]"
              key={c}
              onClick={handleCategoryClick(c)}>
              <p className="text-zinc-400">{c}</p>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default ActionItem;
