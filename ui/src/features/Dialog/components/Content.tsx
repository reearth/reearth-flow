import { DialogContent as DialogContentPrimitive, IconButton } from "@flow/components";
import { DialogType } from "@flow/stores";

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
  const settings = useSettings();

  const instructions = useInstructions();

  const content = tab?.includes("settings")
    ? settings
    : tab?.includes("instructions")
      ? instructions
      : null;

  return content ? (
    <DialogContentPrimitive>
      <div className="flex">
        {content.length > 1 && (
          <div className="flex flex-col gap-6 pr-4 pt-10 border-r border-zinc-800">
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
        <div id="settings-content" className="pl-4">
          {content.find(c => c.id === tab)?.component}
        </div>
      </div>
    </DialogContentPrimitive>
  ) : null;
};

export { DialogContent };
