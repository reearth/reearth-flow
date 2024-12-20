import { useNavigate, useRouterState } from "@tanstack/react-router";
import { useCallback, useMemo } from "react";

import { useDeployment } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { jobs as mockJobs } from "@flow/mock_data/jobsData";
import { useCurrentWorkspace } from "@flow/stores";
import { lastOfUrl as getJobId } from "@flow/utils";

import { RouteOption } from "../WorkspaceLeftPanel";

export default () => {
  const t = useT();
  const navigate = useNavigate();

  const { useGetJobsInfinite } = useDeployment();

  const [currentWorkspace] = useCurrentWorkspace();

  const { pages: jobsPages } = useGetJobsInfinite(currentWorkspace?.id);

  console.log("jobsPages", jobsPages);

  const {
    location: { pathname },
  } = useRouterState();

  const tab = getTab(pathname);

  const selectedJob = useMemo(
    () => mockJobs.find((job) => job.id === tab),
    [tab],
  );

  const handleJobSelect = useCallback(
    (jobId: string) =>
      navigate({
        to: `/workspaces/${currentWorkspace?.id}/jobs/${jobId}`,
      }),
    [currentWorkspace, navigate],
  );

  const jobs = useMemo(
    () =>
      mockJobs.filter((job) => {
        if (tab === "running") return job.status === "running";
        if (tab === "queued") return job.status === "queued";
        if (tab === "completed")
          return job.status === "completed" || job.status === "failed";
        return true;
      }),
    [tab],
  );

  const statusLabels = useMemo(
    () => ({
      completed: t("Completed jobs"),
      running: t("Ongoing jobs"),
      queued: t("Queued jobs"),
      all: t("All jobs"),
    }),
    [t],
  );

  return {
    tab,
    statusLabels,
    selectedJob,
    jobs,
    handleJobSelect,
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
            : getJobId(pathname);
