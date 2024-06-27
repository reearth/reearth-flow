import { DialogContent as DialogContentPrimitive, IconButton } from "@flow/components";
import { DialogType } from "@flow/stores";

import useInstructions from "./instructions/useInstructions";
import useProject from "./project/useProject";
import useSearches from "./searches/useSearches";
import useSettings from "./settings/useSettings";
import useWorkspace from "./workspace/useWorkspace";

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
  const addWorkspace = useWorkspace();
  const addProject = useProject();

  // TODO: Isn't this very hackish?
  const content = tab?.includes("search")
    ? searches
    : tab?.includes("settings")
      ? settings
      : tab?.includes("instructions")
        ? instructions
        : tab?.includes("workspace")
          ? addWorkspace
          : tab?.includes("project")
            ? addProject
            : null;

  const disableClickAway = tab.includes("settings");

  return content ? (
    <DialogContentPrimitive
      className={`${tab === "canvas-search" ? "p-2" : undefined}`}
      size={tab === "canvas-search" ? "md" : undefined}
      position={position}
      hideCloseButton={tab === "canvas-search"}
      overlayBgClass={tab === "canvas-search" ? "bg-black/30" : undefined}
      onPointerDownOutside={e => disableClickAway && e.preventDefault()}
      onEscapeKeyDown={e => disableClickAway && e.preventDefault()}>
      <div className="flex">
        {content.length > 1 && (
          <div className={`flex flex-col gap-4 pr-5 py-6 border-r border-zinc-800`}>
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
