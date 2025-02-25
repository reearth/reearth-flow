import { useNavigate, useRouterState } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { useJob } from "@flow/lib/gql/job";
import { useCurrentWorkspace } from "@flow/stores";
import type { Job } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";
import { lastOfUrl as getJobId } from "@flow/utils";

import { RouteOption } from "../WorkspaceLeftPanel";

export default () => {
  const ref = useRef<HTMLDivElement>(null);
  const navigate = useNavigate();

  const [openJobRunDialog, setOpenJobRunDialog] = useState(false);
  const [currentWorkspace] = useCurrentWorkspace();
  const [currentPage, setCurrentPage] = useState<number>(1);
  const [currentOrder, setCurrentOrder] = useState<OrderDirection>(
    OrderDirection.Desc,
  );
  const { useGetJobs, useCancelJob } = useJob();

  const { page, refetch, isFetching } = useGetJobs(currentWorkspace?.id, {
    page: currentPage,
    orderBy: "completedAt",
    orderDir: currentOrder,
  });

  useEffect(() => {
    refetch();
  }, [currentPage, currentOrder, refetch]);

  const totalPages = page?.totalPages as number;

  const {
    location: { pathname },
  } = useRouterState();

  const tab = getTab(pathname);
  const jobs = page?.jobs;

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

  const handleCancelJob = useCallback(async () => {
    if (!selectedJob) return;
    await useCancelJob(selectedJob.id);
  }, [selectedJob, useCancelJob]);

  return {
    ref,
    jobs,
    selectedJob,
    openJobRunDialog,
    setOpenJobRunDialog,
    handleJobSelect,
    isFetching,
    currentPage,
    setCurrentPage,
    totalPages,
    currentOrder,
    setCurrentOrder,
    handleCancelJob,
  };
};

const getTab = (pathname: string): RouteOption =>
  pathname.includes("all") ? "all" : getJobId(pathname);
