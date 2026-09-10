import {
  BinocularsIcon,
  ProhibitIcon,
  TargetIcon,
} from "@phosphor-icons/react";
import { useState } from "react";

import { IconButton } from "@flow/components";
import type { UserActivity } from "@flow/features/Editor/useUserActivities";
import { useT } from "@flow/lib/i18n";
import { JobStatus, UserDebugRun } from "@flow/types";

const getActivityLabel = (
  activity: UserActivity | undefined,
  t: (key: string, options?: Record<string, unknown>) => string,
): string | undefined => {
  if (!activity) return undefined;
  switch (activity.kind) {
    case "following":
      return t("Following {{user}}", { user: activity.label });
    case "subEditor":
      switch (activity.editor) {
        case "python":
          return t("Python editor: {{field}}", { field: activity.label });
        case "flowExpr":
          return t("Expression editor: {{field}}", { field: activity.label });
        default:
          return t("Value editor: {{field}}", { field: activity.label });
      }
    case "editingNode":
      return activity.label
        ? t("Editing {{node}}", { node: activity.label })
        : t("Editing a node");
    case "workflowVariables":
      return t("Editing workflow variables");
    case "assets":
      return t("Browsing assets");
    case "nodePicker":
      return t("Adding a node");
    case "version":
      return activity.snapshotNumber !== undefined
        ? t("Version history: snapshot {{snapshot}}", {
            snapshot: activity.snapshotNumber,
          })
        : t("Viewing version history");
    case "layout":
      return t("Adjusting layout");
    case "deploy":
      return t("Deploying");
    case "share":
      return t("Sharing the project");
    case "debugVariables":
      return t("Setting up a debug run");
    case "debugRun":
      return activity.status
        ? t("Debug run ({{status}})", { status: activity.status })
        : t("Running a debug job");
    case "viewing":
      return activity.label
        ? t("Viewing {{workflow}}", { workflow: activity.label })
        : undefined;
    default:
      return undefined;
  }
};

type Props = {
  self?: boolean;
  clientId: number;
  userDebugRun?: UserDebugRun;
  userName: string;
  color: string;
  activity?: UserActivity;
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
  activity,
  spotlightUserClientId,
  time,
  onSpotlightUserSelect,
  onSpotlightUserDeselect,
  onDebugRunJoin,
}) => {
  const isSpotlighted = spotlightUserClientId === clientId;
  const t = useT();
  const activityLabel = getActivityLabel(activity, t);
  const [isHovered, setIsHovered] = useState(false);
  const getDebugRunStatusLabel = (
    status: JobStatus | undefined,
    t: (key: string) => string,
  ) => {
    switch (status) {
      case "completed":
        return t("Completed");
      case "running":
        return t("Running");
      case "cancelled":
        return t("Cancelled");
      case "failed":
        return t("Failed");
      case "queued":
        return t("Queued");
      default:
        return t("Unknown");
    }
  };
  return (
    <div
      className="flex items-center gap-2 rounded-lg p-1 hover:bg-primary"
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}>
      <div
        className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full ring-2 ring-secondary/20"
        style={{ backgroundColor: color }}>
        <span className="text-sm font-medium text-white select-none">
          {userName.charAt(0).toUpperCase()}
          {userName.charAt(1)}
        </span>
      </div>
      <div className="flex min-w-0 flex-col">
        <span className="truncate text-sm select-none dark:font-light">
          {userName}
        </span>
        {activityLabel && (
          <span
            className="truncate text-xs opacity-55 select-none dark:font-light"
            title={activityLabel}>
            {activityLabel}
          </span>
        )}

        <div className="flex items-center gap-2">
          {(time || userDebugRun) && (
            <div className="flex items-center gap-2">
              {time && (
                <div className="flex items-center gap-0.5">
                  <span className="text-sm opacity-55 dark:font-light">
                    {getDebugRunStatusLabel(userDebugRun?.status, t)}
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
