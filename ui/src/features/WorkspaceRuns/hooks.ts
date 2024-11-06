import { useNavigate, useRouterState } from "@tanstack/react-router";
import { useCallback, useMemo } from "react";

import { useT } from "@flow/lib/i18n";
import { runs as mockRuns } from "@flow/mock_data/runsData";
import { useCurrentWorkspace } from "@flow/stores";
import { lastOfUrl as getRunId } from "@flow/utils";

import { RouteOption } from "../WorkspaceLeftPanel";

export default () => {
  const t = useT();
  const navigate = useNavigate();

  const [currentWorkspace] = useCurrentWorkspace();

  const {
    location: { pathname },
  } = useRouterState();

  const tab = getTab(pathname);

  const selectedRun = useMemo(
    () => mockRuns.find((run) => run.id === tab),
    [tab],
  );

  const handleRunSelect = useCallback(
    (runId: string) =>
      navigate({
        to: `/workspaces/${currentWorkspace?.id}/runs/${runId}`,
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

  return {
    tab,
    statusLabels,
    selectedRun,
    runs,
    handleRunSelect,
  };
};

const getTab = (pathname: string): RouteOption =>
  pathname.includes("running")
    ? "running"
    : pathname.includes("new")
      ? "new"
      : pathname.includes("queued")
        ? "queued"
        : pathname.includes("completed")
          ? "completed"
          : pathname.includes("all")
            ? "all"
            : getRunId(pathname);
