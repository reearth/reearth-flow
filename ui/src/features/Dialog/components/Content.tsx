import { DialogContent as DialogContentPrimitive, IconButton } from "@flow/components";
import { DialogType, useCurrentProject } from "@flow/stores";

import useInits from "./inits/useInits";
import useInstructions from "./instructions/useInstructions";
import useSettings from "./settings/useSettings";

export type DialogContentType = {
  id: DialogType;
  title: string;
  icon: React.ReactNode;
  component: React.ReactNode;
};

type Props = {
  tab: DialogType;
  onTabChange: (tab: DialogType) => void;
};

const DialogContent: React.FC<Props> = ({ tab, onTabChange }) => {
  const [currentProject] = useCurrentProject();
  const inits = useInits();
  const settings = useSettings();
  const instructions = useInstructions();

  const content = tab?.includes("init")
    ? inits
    : tab?.includes("settings")
      ? settings
      : tab?.includes("instructions")
        ? instructions
        : null;

  const disableClickaway = tab.includes("settings") || (tab === "welcome-init" && !currentProject);

  return content ? (
    <DialogContentPrimitive
      size={tab === "welcome-init" ? "2xl" : undefined}
      hideCloseButton={tab === "welcome-init"}
      onPointerDownOutside={e => disableClickaway && e.preventDefault()}
      onEscapeKeyDown={e => disableClickaway && e.preventDefault()}>
      <div className="flex">
        {content.length > 1 && (
          <div className="flex flex-col pr-8 py-6 border-r border-zinc-800">
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
