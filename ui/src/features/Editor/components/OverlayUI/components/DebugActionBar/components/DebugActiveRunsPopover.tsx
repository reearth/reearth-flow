import { BinocularsIcon } from "@phosphor-icons/react";

import {
  IconButton,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  showPopover: string | undefined;
  onShowDebugRunsPopover: () => void;
  onDebugRunStart: () => Promise<void>;
  onPopoverClose: () => void;
};

const DebugActiveRunsPopover: React.FC<Props> = ({
  showPopover,
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
          <IconButton
            className="shrink-0"
            disabled={true}
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
            {/* {activeUsersDebugRuns && activeUsersDebugRuns.length >= 1 && (
              <ScrollArea>
                <div className="flex max-h-[250px] flex-col gap-2">
                  <div className="flex flex-col gap-2">
                    {activeUsersDebugRuns.map((user) => {
                      if (!user.debugRun) return null;
                      const timeSinceStart = Math.max(
                        0,
                        Date.now() - user.debugRun.startedAt,
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
                          key={user?.debugRun?.jobId}
                          clientId={user.clientId}
                          userDebugRun={user.debugRun}
                          userName={user.userName}
                          color={user.color}
                          time={timeAgo}
                        />
                      );
                    })}
                  </div>
                </div>
              </ScrollArea>
            )} */}
          </div>
        )}
      </PopoverContent>
    </Popover>
  );
};

export default DebugActiveRunsPopover;
