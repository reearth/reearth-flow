import { useNavigate, useRouterState } from "@tanstack/react-router";
import { useCallback, useMemo, useState } from "react";

import { useJob } from "@flow/lib/gql/job";
import { useCurrentWorkspace } from "@flow/stores";
import type { Job } from "@flow/types";
import { lastOfUrl as getJobId } from "@flow/utils";

import usePagination from "../hooks/usePagination";
import { RouteOption } from "../WorkspaceLeftPanel";

const JOBS_FETCH_RATE = 15;
export default () => {
  const navigate = useNavigate();

  const [openJobRunDialog, setOpenJobRunDialog] = useState(false);
  const [currentWorkspace] = useCurrentWorkspace();
  const [currentPage, setCurrentPage] = useState<number>(0);
  const { useGetJobsInfinite } = useJob();

  const { pages, hasNextPage, isFetchingNextPage, fetchNextPage } =
    useGetJobsInfinite(currentWorkspace?.id, JOBS_FETCH_RATE);

  const {
    location: { pathname },
  } = useRouterState();

  const tab = getTab(pathname);

  const jobs: Job[] | undefined = useMemo(
    () => pages?.[currentPage]?.jobs,
    [pages, currentPage],
  );

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

  const { totalPages, handleNextPage, handlePrevPage, canGoNext } =
    usePagination<Job>(
      JOBS_FETCH_RATE,
      hasNextPage,
      isFetchingNextPage,
      pages,
      fetchNextPage,
      currentPage,
      setCurrentPage,
    );

  return {
    jobs,
    selectedJob,
    totalPages,
    currentPage,
    hasNextPage: canGoNext,
    isFetchingNextPage,
    handleNextPage,
    handlePrevPage,
    openJobRunDialog,
    setOpenJobRunDialog,
    handleJobSelect,
  };
};

const getTab = (pathname: string): RouteOption =>
  pathname.includes("all") ? "all" : getJobId(pathname);
