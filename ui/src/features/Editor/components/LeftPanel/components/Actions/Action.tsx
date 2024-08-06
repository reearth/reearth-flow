import { Lightning } from "@phosphor-icons/react";
import { type DragEvent, memo } from "react";
import { createRoot } from "react-dom/client";

import type { Action } from "@flow/types";

type Props = Action & {
  selected: boolean;
  onSelect: () => void;
};

const ActionComponent: React.FC<Props> = ({
  name,
  type,
  description,
  categories,
  selected,
  onSelect,
}) => {
  const onDragStart = (
    event: DragEvent<HTMLDivElement>,
    actionName: string
  ) => {
    event.dataTransfer.setData("application/reactflow", actionName);
    event.dataTransfer.effectAllowed = "move";
    const dragPreviewContainer = document.createElement("div");
    dragPreviewContainer.style.position = "absolute";
    dragPreviewContainer.style.top = "-1000px"; // Move it offscreen to hide it

    const root = createRoot(dragPreviewContainer);
    root.render(
      <div className="flex size-12 rounded bg-secondary">
        <div
          className={`flex w-full justify-center rounded align-middle  ${type === "reader" ? "bg-[#164E63]/60" : type === "writer" ? "bg-[#635116]/60" : "bg-[#631628]/60"}`}
        >
          <Lightning className="self-center" />
        </div>
      </div>
    );

    document.body.appendChild(dragPreviewContainer);
    event.dataTransfer.setDragImage(dragPreviewContainer, 10, 10);

    // Clean up the container after the drag starts
    setTimeout(() => {
      root.unmount();
      document.body.removeChild(dragPreviewContainer);
    }, 0);
  };

  return (
    <div
      className={`group cursor-pointer rounded px-2 ${selected ? "bg-primary text-accent-foreground" : "hover:bg-primary hover:text-accent-foreground"}`}
      onMouseDown={onSelect}
      onDragStart={(e) => onDragStart(e, name)}
      draggable
    >
      <div className="flex w-full justify-between gap-1 py-2">
        <div className="w-3/5 self-center break-words text-sm">
          <p className="self-center text-zinc-200">{name}</p>
        </div>
        <div
          className={`self-center rounded border bg-popover p-1 align-middle`}
        >
          <p className="self-center text-xs capitalize text-zinc-200">{type}</p>
        </div>
      </div>
      <div className="group-hover:block">
        <div className="mb-2 text-xs leading-[0.85rem]">{description}</div>
        <div className="flex flex-wrap gap-1 text-xs ">
          {categories.map((c) => (
            <div className="rounded border bg-popover p-[2px]" key={c}>
              <p className="text-zinc-400">{c}</p>
            </div>
          ))}
        </div>
      </div>
      <div className="border-b pb-2" />
    </div>
  );
};

export default memo(ActionComponent);
