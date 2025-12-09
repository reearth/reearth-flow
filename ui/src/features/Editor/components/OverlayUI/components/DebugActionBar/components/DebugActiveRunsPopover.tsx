import { BinocularsIcon } from "@phosphor-icons/react";

import {
  IconButton,
  Popover,
  PopoverContent,
  PopoverTrigger,
  ScrollArea,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { UserDebug } from "@flow/types";

import CollaborationCard from "../../Collaboration/CollaborationCard";

type Props = {
  activeDebugRuns?: UserDebug[];
  showPopover: string | undefined;
  onDebugRunJoin?: (jobId: string, userName: string) => void;
  onShowDebugRunsPopover: () => void;
  onDebugRunStart: () => Promise<void>;
  onPopoverClose: () => void;
};

const DebugActiveRunsPopover: React.FC<Props> = ({
  activeDebugRuns,
  showPopover,
  onDebugRunJoin,
  onShowDebugRunsPopover,
  onPopoverClose,
}) => {
  const t = useT();

  return (
    <Popover
      open={showPopover === "debugRuns"}
      onOpenChange={(open) => {
        if (!open) onPopoverClose();
      }}>
      <PopoverTrigger asChild>
        <div className="relative">
          {activeDebugRuns && activeDebugRuns.length >= 1 && (
            <div className="absolute top-1.5 right-0.5 h-2 w-2 shrink-0 items-center justify-center rounded-full bg-green-400 " />
          )}
          <IconButton
            className="shrink-0"
            disabled={activeDebugRuns && activeDebugRuns.length === 0}
            tooltipText={t("Active Debug Runs")}
            tooltipOffset={6}
            icon={<BinocularsIcon weight="thin" size={18} />}
            onClick={onShowDebugRunsPopover}
          />
        </div>
      </PopoverTrigger>
      <PopoverContent
        sideOffset={8}
        collisionPadding={5}
        className="bg-primary/50 backdrop-blur">
        {showPopover === "debugRuns" && (
          <div className="flex flex-col  gap-2 p-4">
            <div className="flex justify-between gap-2">
              <h4 className="text-md flex items-center gap-2 self-center rounded-t-lg leading-none tracking-tight dark:font-thin">
                <BinocularsIcon weight="thin" size={18} />
                {t("Active Debug Runs")}
              </h4>
            </div>
            {activeDebugRuns && activeDebugRuns.length >= 1 && (
              <ScrollArea>
                <div className="flex max-h-[250px] flex-col gap-2">
                  <div className="flex flex-col gap-2">
                    {activeDebugRuns.map((debug) => {
                      const timeSinceStart = Math.max(
                        0,
                        Date.now() - debug.startedAt,
                      );
                      let timeAgo;
                      if (timeSinceStart < 60000) {
                        timeAgo = t("just now");
                      } else if (timeSinceStart < 3600000) {
                        timeAgo = t("{{minutes}}m ago", {
                          minutes: Math.floor(timeSinceStart / 60000),
                        });
                      } else {
                        timeAgo = t("{{hours}}h ago", {
                          hours: Math.floor(timeSinceStart / 3600000),
                        });
                      }
                      return (
                        <CollaborationCard
                          key={debug.jobId}
                          clientId={debug.clientId}
                          userName={debug.userName}
                          color={debug.color}
                          time={timeAgo}
                          onDebugRunJoin={() => {
                            onPopoverClose();
                            onDebugRunJoin?.(debug.jobId, debug.userName);
                          }}
                        />
                      );
                    })}
                  </div>
                </div>
              </ScrollArea>
            )}
          </div>
        )}
      </PopoverContent>
    </Popover>
  );
};

export default DebugActiveRunsPopover;
