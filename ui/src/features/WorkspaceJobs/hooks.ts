import { useNavigate, useRouterState } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { useJob } from "@flow/lib/gql/job";
import { useCurrentWorkspace } from "@flow/stores";
import type { Job } from "@flow/types";
import { lastOfUrl as getJobId } from "@flow/utils";

import { RouteOption } from "../WorkspaceLeftPanel";

export default () => {
  const ref = useRef<HTMLDivElement>(null);
  const navigate = useNavigate();

  const [openJobRunDialog, setOpenJobRunDialog] = useState(false);
  const [currentWorkspace] = useCurrentWorkspace();
  const [currentPage, setCurrentPage] = useState<number>(1);

  const { useGetJobs } = useJob();
  const JOBS_FETCH_RATE_PER_PAGE = 15;
  const { pages, refetch } = useGetJobs(currentWorkspace?.id, {
    pageSize: JOBS_FETCH_RATE_PER_PAGE,
    page: currentPage,
  });

  useEffect(() => {
    refetch();
  }, [currentPage, refetch]);
  const totalPages = pages?.totalPages as number;

  const {
    location: { pathname },
  } = useRouterState();

  const tab = getTab(pathname);
  const jobs = pages?.jobs;

  const selectedJob = useMemo(
    () => jobs?.find((job) => job.id === tab),
    [tab, jobs],
  );

  const handleJobSelect = useCallback(
    (job: Job) =>
      navigate({
        to: `/workspaces/${currentWorkspace?.id}/jobs/${job.id}`,
      }),
    [currentWorkspace, navigate],
  );

  return {
    ref,
    jobs,
    selectedJob,
    openJobRunDialog,
    setOpenJobRunDialog,
    handleJobSelect,
    currentPage,
    setCurrentPage,
    totalPages,
    JOBS_FETCH_RATE_PER_PAGE,
  };
};

const getTab = (pathname: string): RouteOption =>
  pathname.includes("all") ? "all" : getJobId(pathname);
