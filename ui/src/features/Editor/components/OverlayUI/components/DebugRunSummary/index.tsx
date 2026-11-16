import {
  BugIcon,
  CaretDownIcon,
  WarningCircleIcon,
} from "@phosphor-icons/react";
import { memo, useMemo, useState } from "react";

import { useJob } from "@flow/lib/gql/job";
import { useSubscription } from "@flow/lib/gql/subscriptions/useSubscription";
import { useLang, useT } from "@flow/lib/i18n";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { cn } from "@flow/lib/utils";
import { useCurrentProject } from "@flow/stores";
import type { AnyWorkflowVariable, JobStatus } from "@flow/types";

import { toDebugRunVariables } from "./debugRunVariables";
import { formatRunTime } from "./formatRunTime";
import VariablesSection from "./VariablesSection";

type Props = {
  className?: string;
  workflowVariableDefaults?: AnyWorkflowVariable[];
};

const isActive = (status?: JobStatus) =>
  status === "running" || status === "queued";

// The current debug run at a glance, kept to what the action bar above does
// not already show: when the run happened, whether the workflow has changed
// since, whose run it is and the workflow variables it used.
// Styled as part of the overlay, but a step quieter than the action bar.
const DebugRunSummary: React.FC<Props> = ({
  className,
  workflowVariableDefaults,
}) => {
  const t = useT();
  const lang = useLang();
  const [expanded, setExpanded] = useState(true);
  const [currentProject] = useCurrentProject();
  const { value: debugRunState } = useIndexedDB("debugRun");
  const { useGetJob } = useJob();

  const debugJobState = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id),
    [debugRunState, currentProject],
  );

  const { job } = useGetJob(debugJobState?.jobId);
  // Reads the status the action bar's subscription keeps up to date.
  const { data: liveStatus } = useSubscription(
    "GetSubscribedJobStatus",
    debugJobState?.jobId,
    !debugJobState?.jobId,
  );
  const status: JobStatus | undefined = liveStatus ?? job?.status;

  const variables = useMemo(
    () =>
      toDebugRunVariables({
        recorded: job?.variables,
        local: debugJobState?.variables,
        runDefaults: debugJobState?.variableDefaults,
        defaults: workflowVariableDefaults,
      }),
    [
      job?.variables,
      debugJobState?.variables,
      debugJobState?.variableDefaults,
      workflowVariableDefaults,
    ],
  );

  if (!debugJobState) return null;

  // The status in words only where the action bar's dot leaves it unclear.
  const statusLabel =
    status === "failed"
      ? { text: t("Failed"), className: "text-destructive" }
      : status === "cancelled"
        ? { text: t("Cancelled"), className: "text-warning" }
        : status === "queued"
          ? { text: t("Queued"), className: "" }
          : undefined;

  const finishedAt =
    status && !isActive(status)
      ? formatRunTime(job?.completedAt, lang)
      : undefined;
  const startedAt = formatRunTime(job?.startedAt, lang);
  const timeLabel = finishedAt
    ? t("Finished {{time}}", { time: finishedAt })
    : startedAt
      ? t("Started {{time}}", { time: startedAt })
      : undefined;

  // Same rule as the action bar: a run still going cannot be out of date yet.
  const isStale = !!debugJobState.isRunStale && !isActive(status);
  const staleCount = debugJobState.staleNodeIds?.length ?? 0;

  const notes = [
    debugJobState.joinedUserName
      ? t("{{userName}}'s run", { userName: debugJobState.joinedUserName })
      : undefined,
    debugJobState.startNodeId ? t("From selected action") : undefined,
  ].filter((note): note is string => !!note);

  // Collapsing only hides the variables, so it is offered only when there are some.
  const canCollapse = variables.length > 0;

  const header = (
    <>
      <BugIcon weight="light" size={16} className="shrink-0" />
      <span className="shrink-0 text-xs dark:font-light">{t("Debug Run")}</span>
      {statusLabel && (
        <span className={cn("shrink-0 text-[10px]", statusLabel.className)}>
          {statusLabel.text}
        </span>
      )}
      {isStale && (
        <WarningCircleIcon
          weight="light"
          size={14}
          className="shrink-0 text-warning"
          aria-label={t("Edited since this run")}
        />
      )}
      <span className="flex-1" />
      {timeLabel && (
        <span className="truncate text-[10px] tabular-nums">{timeLabel}</span>
      )}
    </>
  );

  return (
    <div
      className={cn(
        // Capped to end above the debug panel at its default height (45vh
        // from bottom-2), below the action bar (top-2 + 42px + gap).
        "flex max-h-[calc(55vh-72px)] flex-col rounded-xl border border-border/50 bg-secondary/40 p-1 text-muted-foreground opacity-80 shadow-sm shadow-[black]/5 backdrop-blur-xs transition-opacity select-none hover:opacity-100 dark:border-primary/50",
        className,
      )}>
      {canCollapse ? (
        <button
          type="button"
          className="group flex h-8 w-full shrink-0 cursor-pointer items-center gap-2 rounded-lg px-2 text-left hover:bg-accent/30 hover:text-foreground"
          aria-expanded={expanded}
          aria-label={expanded ? t("Collapse") : t("Expand")}
          onClick={() => setExpanded((e) => !e)}>
          {header}
          <CaretDownIcon
            size={12}
            className={cn(
              // Only on hover or focus: collapsing is there, but rarely needed.
              "shrink-0 opacity-0 transition group-hover:opacity-100 group-focus-visible:opacity-100",
              expanded && "rotate-180",
            )}
          />
        </button>
      ) : (
        <div className="flex h-8 w-full shrink-0 items-center gap-2 px-2">
          {header}
        </div>
      )}
      {/* Short and worth seeing, so not hidden by collapsing. */}
      {(isStale || notes.length > 0) && (
        <div className="flex shrink-0 flex-col gap-0.5 px-2 pb-1.5 text-[10px]">
          {isStale && (
            <p className="truncate">
              <span className="text-warning">{t("Edited since this run")}</span>
              {staleCount > 0 &&
                ` · ${t("{{count}} actions out of date", { count: staleCount })}`}
            </p>
          )}
          {notes.length > 0 && <p className="truncate">{notes.join(" · ")}</p>}
        </div>
      )}
      {canCollapse && expanded && (
        <div className="min-h-0 overflow-y-auto">
          <VariablesSection variables={variables} />
        </div>
      )}
    </div>
  );
};

export default memo(DebugRunSummary);
