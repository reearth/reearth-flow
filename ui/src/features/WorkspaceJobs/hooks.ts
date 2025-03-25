import { useNavigate } from "@tanstack/react-router";
import { useCallback, useEffect, useRef, useState } from "react";

import { useJob } from "@flow/lib/gql/job";
import { useCurrentWorkspace } from "@flow/stores";
import type { Job } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";

export default () => {
  const ref = useRef<HTMLDivElement>(null);
  const navigate = useNavigate();

  const [openJobRunDialog, setOpenJobRunDialog] = useState(false);
  const [currentWorkspace] = useCurrentWorkspace();
  const [currentPage, setCurrentPage] = useState<number>(1);
  const [currentOrder, setCurrentOrder] = useState<OrderDirection>(
    OrderDirection.Desc,
  );
  const { useGetJobs } = useJob();

  const { page, refetch, isFetching } = useGetJobs(currentWorkspace?.id, {
    page: currentPage,
    orderDir: currentOrder,
  });

  useEffect(() => {
    (async () => {
      await refetch();
    })();
  }, [currentPage, currentOrder, refetch]);

  const totalPages = page?.totalPages as number;

  const jobs = page?.jobs;

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
    openJobRunDialog,
    isFetching,
    currentPage,
    totalPages,
    currentOrder,
    setOpenJobRunDialog,
    handleJobSelect,
    setCurrentPage,
    setCurrentOrder,
  };
};
