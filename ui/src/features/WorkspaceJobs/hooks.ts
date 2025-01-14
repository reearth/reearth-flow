import { useNavigate, useRouterState } from "@tanstack/react-router";
import { useCallback, useMemo } from "react";

import { useJob } from "@flow/lib/gql/job";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import { Job } from "@flow/types";
import { lastOfUrl as getJobId } from "@flow/utils";

import { RouteOption } from "../WorkspaceLeftPanel";

export default () => {
  const t = useT();
  const navigate = useNavigate();

  const { useGetJobsInfinite } = useJob();

  const [currentWorkspace] = useCurrentWorkspace();

  // const { pages, hasNextPage, isFetching, fetchNextPage } = useGetJobsInfinite(
  const { pages } = useGetJobsInfinite(currentWorkspace?.id); // TODO: Add pagination

  const rawJobs: Job[] | undefined = useMemo(
    () =>
      pages?.reduce((jobs, page) => {
        if (page?.jobs) {
          jobs.push(...page.jobs);
        }
        return jobs;
      }, [] as Job[]),
    [pages],
  );

  const {
    location: { pathname },
  } = useRouterState();

  const tab = getTab(pathname);

  const selectedJob = useMemo(
    () => rawJobs?.find((job) => job.id === tab),
    [tab, rawJobs],
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
      rawJobs?.filter((job) => {
        if (tab === "running") return job.status === "running";
        if (tab === "queued") return job.status === "queued";
        if (tab === "completed")
          return job.status === "completed" || job.status === "failed";
        return true;
      }),
    [tab, rawJobs],
  );

  const statusLabels = useMemo(
    () => ({
      completed: t("Completed jobs"),
      running: t("Ongoing jobs"),
      queued: t("Queued jobs"),
      all: t("Jobs"),
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
