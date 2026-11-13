import {
  BinocularsIcon,
  ProhibitIcon,
  TargetIcon,
} from "@phosphor-icons/react";
import { useState } from "react";

import { IconButton, PresenceAvatar } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { UserDebugRun } from "@flow/types";
import { getJobStatusLabel } from "@flow/utils";

type Props = {
  self?: boolean;
  clientId: number;
  userDebugRun?: UserDebugRun;
  userName: string;
  color: string;
  photoURL?: string;
  spotlightUserClientId?: number | null;
  time?: string;
  onSpotlightUserSelect?: (clientId: number) => void;
  onSpotlightUserDeselect?: () => void;
  onDebugRunJoin?: () => void;
};

const CollaborationCard: React.FC<Props> = ({
  self,
  clientId,
  userDebugRun,
  userName,
  color,
  photoURL,
  spotlightUserClientId,
  time,
  onSpotlightUserSelect,
  onSpotlightUserDeselect,
  onDebugRunJoin,
}) => {
  const isSpotlighted = spotlightUserClientId === clientId;
  const t = useT();
  const [isHovered, setIsHovered] = useState(false);
  return (
    <div
      className="flex items-center gap-2 rounded-lg p-1 hover:bg-primary"
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}>
      <PresenceAvatar user={{ userName, color, photoURL }} size="md" />
      <div className="flex min-w-0 flex-col">
        <span className="truncate text-sm select-none dark:font-light">
          {userName}
        </span>

        <div className="flex items-center gap-2">
          {(time || userDebugRun) && (
            <div className="flex items-center gap-2">
              {time && (
                <div className="flex items-center gap-0.5">
                  <span className="text-sm opacity-55 dark:font-light">
                    {getJobStatusLabel(userDebugRun?.status, t)}
                  </span>
                  <span className="text-sm opacity-55 dark:font-light">
                    {t("({{time}})", { time })}
                  </span>
                </div>
              )}
              <div
                className={`${
                  userDebugRun?.status === "completed"
                    ? "bg-success"
                    : userDebugRun?.status === "running"
                      ? "active-node-status"
                      : userDebugRun?.status === "cancelled"
                        ? "bg-warning"
                        : userDebugRun?.status === "failed"
                          ? "bg-destructive"
                          : userDebugRun?.status === "queued"
                            ? "queued-node-status"
                            : "bg-secondary"
                } size-3 rounded-full`}
              />
            </div>
          )}
        </div>
      </div>
      <div className="ml-auto">
        {isHovered && onSpotlightUserSelect && !isSpotlighted && !self && (
          <IconButton
            className="h-8"
            tooltipText={t("Spotlight User")}
            icon={<TargetIcon size={14} />}
            onClick={() => onSpotlightUserSelect(clientId)}
          />
        )}
        {isSpotlighted && onSpotlightUserDeselect && !self && (
          <IconButton
            className="h-8"
            tooltipText={t("Remove Spotlight")}
            icon={<ProhibitIcon size={14} />}
            onClick={onSpotlightUserDeselect}
          />
        )}
        {isHovered && onDebugRunJoin && !self && (
          <IconButton
            className="h-8"
            tooltipText={t("View Debug Run")}
            icon={<BinocularsIcon size={14} />}
            onClick={onDebugRunJoin}
          />
        )}
      </div>
    </div>
  );
};

export default CollaborationCard;
