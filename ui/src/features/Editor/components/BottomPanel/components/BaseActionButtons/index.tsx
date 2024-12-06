import { memo } from "react";

import { ContentID } from "../Contents";

export type PanelContent = {
  id: ContentID;
  title?: string;
  icon?: React.ReactNode;
};

const BaseActionButtons: React.FC<{
  panelContentOptions?: PanelContent[];
  selectedId?: ContentID;
  onSelection?: (id: ContentID) => void;
}> = memo(({ panelContentOptions, selectedId, onSelection }) => {
  return (
    <>
      {panelContentOptions?.map((content, idx) => (
        <div
          key={content.id}
          className={`flex h-4/5 min-w-[140px] cursor-pointer items-center justify-center gap-2 rounded hover:bg-popover hover:text-popover-foreground ${
            (!selectedId && idx === 0) || selectedId === content.id
              ? "bg-popover text-popover-foreground"
              : "bg-card"
          }`}
          onClick={() => onSelection?.(content.id)}>
          {content.icon}
          <p className="text-sm dark:font-thin">{content.title}</p>
        </div>
      ))}
    </>
  );
});

export { BaseActionButtons };
