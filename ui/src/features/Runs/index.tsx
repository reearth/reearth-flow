import { useNavigate, useRouterState } from "@tanstack/react-router";
import { useCallback, useMemo } from "react";

import { useT } from "@flow/lib/i18n";
import { runs as mockRuns } from "@flow/mock_data/runsData";
import { useCurrentWorkspace } from "@flow/stores";
import type { Run } from "@flow/types";

import { RouteOption } from "../WorkspaceLeftPanel";

import { NewRun, StatusContent, RunDetails } from "./components";

type Status = "running" | "queued" | "completed";

const Runs: React.FC = () => {
  const t = useT();

  const {
    location: { pathname },
  } = useRouterState();

  const tab: RouteOption = pathname.includes("running")
    ? "running"
    : pathname.includes("new")
      ? "new"
      : pathname.includes("queued")
        ? "queued"
        : pathname.includes("completed")
          ? "completed"
          : "all";

  const navigate = useNavigate();
  const [currentWorkspace] = useCurrentWorkspace();

  const selectedRun = useMemo(
    () => mockRuns.find((run) => run.id === tab),
    [tab],
  );

  const handleRunSelect = useCallback(
    (run: Run) =>
      navigate({
        to: `/workspaces/${currentWorkspace?.id}/runs/${run.id}`,
      }),
    [currentWorkspace, navigate],
  );

  const runs = useMemo(
    () =>
      mockRuns.filter((run) => {
        if (tab === "running") return run.status === "running";
        if (tab === "queued") return run.status === "queued";
        if (tab === "completed")
          return run.status === "completed" || run.status === "failed";
        return true;
      }),
    [tab],
  );

  const statusLabels = useMemo(
    () => ({
      completed: t("Completed runs"),
      running: t("Ongoing runs"),
      queued: t("Queued runs"),
      all: t("All runs"),
    }),
    [t],
  );

  return (
    <div className="flex-1">
      {tab === "new" ? (
        <NewRun />
      ) : isList(tab) ? (
        <StatusContent
          label={statusLabels[tab as Status]}
          runs={runs}
          onRunSelect={handleRunSelect}
        />
      ) : (
        <RunDetails selectedRun={selectedRun} />
      )}
    </div>
  );
};

export { Runs };

function isList(value: string) {
  return !!(
    value === "running" ||
    value === "queued" ||
    value === "completed" ||
    value === "all"
  );
}
