import { useNavigate, useRouterState } from "@tanstack/react-router";
import { useCallback, useMemo } from "react";

import { useT } from "@flow/lib/i18n";
import { runs as mockRuns } from "@flow/mock_data/runsData";
import { useCurrentWorkspace } from "@flow/stores";
import { Run } from "@flow/types";

import { RouteOption } from "../WorkspaceLeftPanel";

export default () => {
  const t = useT();

  const {
    location: { pathname },
  } = useRouterState();

  const tab = getTab(pathname);

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

const getRunId = (url: string) => {
  const parts = url.split("/");
  return parts[parts.length - 1];
};
