import { BugIcon, ListDashesIcon } from "@phosphor-icons/react";

import { IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";

export type DebugLogsView = "logs" | "diagnostics";

type Props = {
  view: DebugLogsView;
  diagnosticsCount: number;
  /** Whether the worst diagnostic present is an error or fatal, not a warning. */
  hasBlocking: boolean;
  onViewChange: (view: DebugLogsView) => void;
};

/**
 * Switches the panel between the free-text log stream and structured
 * diagnostics.
 *
 * The two are separate views rather than one table on purpose: `Diagnostic`
 * carries no timestamp, so the two cannot share a sort order, and their columns
 * barely overlap. A diagnostic row is also often an aggregate of many features
 * where a log line is a single event, which one blended list would misrepresent.
 */
const ViewSwitch: React.FC<Props> = ({
  view,
  diagnosticsCount,
  hasBlocking,
  onViewChange,
}) => {
  const t = useT();

  return (
    <div className="flex shrink-0 items-center gap-1">
      <IconButton
        size="icon"
        variant={view === "logs" ? "default" : "outline"}
        tooltipText={t("Workflow Logs")}
        onClick={() => onViewChange("logs")}
        icon={<ListDashesIcon />}
      />
      <div className="relative">
        <IconButton
          size="icon"
          variant={view === "diagnostics" ? "default" : "outline"}
          disabled={!diagnosticsCount}
          tooltipText={
            diagnosticsCount
              ? t("Diagnostics ({{count}})", { count: diagnosticsCount })
              : t("No diagnostics for this run")
          }
          onClick={() => onViewChange("diagnostics")}
          icon={<BugIcon />}
        />
        {diagnosticsCount > 0 && (
          <span
            // Count tells you how much, colour tells you whether to care:
            // warnings alone should not read as a failure.
            className={`pointer-events-none absolute -top-1 -right-1.5 flex h-4 min-w-4 items-center justify-center rounded-full px-1 text-[10px] leading-none font-medium ${
              hasBlocking
                ? "bg-destructive text-destructive-foreground"
                : "bg-warning text-warning-foreground"
            }`}>
            {diagnosticsCount > 99 ? "99+" : diagnosticsCount}
          </span>
        )}
      </div>
    </div>
  );
};

export default ViewSwitch;
