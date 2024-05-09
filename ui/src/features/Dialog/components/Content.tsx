import { DialogContent as DialogContentPrimitive, IconButton } from "@flow/components";
import { DialogType } from "@flow/stores";

import useInstructions from "./instructions/useInstructions";
import useSearches from "./searches/useSearches";
import useSettings from "./settings/useSettings";

export type DialogContentType = {
  id: DialogType;
  title: string;
  icon: React.ReactNode;
  component: React.ReactNode;
};

type Props = {
  tab: DialogType;
  position?: "center" | "top";
  onTabChange: (tab: DialogType) => void;
};

const DialogContent: React.FC<Props> = ({ tab, position, onTabChange }) => {
  const searches = useSearches();
  const settings = useSettings();
  const instructions = useInstructions();

  const content = tab?.includes("search")
    ? searches
    : tab?.includes("settings")
      ? settings
      : tab?.includes("instructions")
        ? instructions
        : null;

  const disableClickaway = tab.includes("settings");

  return content ? (
    <DialogContentPrimitive
      className={`${tab === "canvas-search" ? "p-2" : undefined}`}
      size={tab === "canvas-search" ? "md" : undefined}
      position={position}
      hideCloseButton={tab === "canvas-search"}
      overlayBgClass={tab === "canvas-search" ? "bg-black/30" : undefined}
      onPointerDownOutside={e => disableClickaway && e.preventDefault()}
      onEscapeKeyDown={e => disableClickaway && e.preventDefault()}>
      <div className="flex">
        {content.length > 1 && (
          <div className={`flex flex-col pr-8 py-6 border-r border-zinc-800`}>
            {content.map(c => (
              <IconButton
                key={c.id}
                className={`${tab === c.id ? "bg-zinc-800" : undefined}`}
                tooltipText={c.title}
                tooltipPosition="left"
                tooltipOffset={20}
                icon={c.icon}
                onClick={() => onTabChange?.(c.id)}
              />
            ))}
          </div>
        )}
        <div className="w-full">{content.find(c => c.id === tab)?.component}</div>
      </div>
    </DialogContentPrimitive>
  ) : null;
};

export { DialogContent };
