import { useNavigate } from "@tanstack/react-router";
import { useCallback, useRef, useState } from "react";

import { usePagination } from "@flow/hooks";
import { useJob } from "@flow/lib/gql/job";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import { JobOrderBy, type Job } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";

export default () => {
  const ref = useRef<HTMLDivElement>(null);
  const navigate = useNavigate();
  const t = useT();
  const [openJobRunDialog, setOpenJobRunDialog] = useState(false);
  const [currentWorkspace] = useCurrentWorkspace();
  const { useGetJobs } = useJob();

  const {
    page,
    totalPages,
    isFetching,
    currentPage,
    currentSortValue,
    searchTerm,
    isDebouncingSearch,
    setCurrentPage,
    setCurrentOrderDir,
    setSearchTerm,
    handleSortChange,
  } = usePagination({
    useDataQuery: useGetJobs,
    workspaceId: currentWorkspace?.id,
    defaultOrderBy: JobOrderBy.StartedAt,
  });

  const jobs = page?.jobs;

  const sortOptions = [
    {
      value: `${JobOrderBy.StartedAt}_${OrderDirection.Desc}`,
      label: t("Recently Started"),
    },
    {
      value: `${JobOrderBy.StartedAt}_${OrderDirection.Asc}`,
      label: t("Oldest Started"),
    },
    {
      value: `${JobOrderBy.CompletedAt}_${OrderDirection.Desc}`,
      label: t("Recently Completed"),
    },
    {
      value: `${JobOrderBy.CompletedAt}_${OrderDirection.Asc}`,
      label: t("Oldest Completed"),
    },
  ];

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
    currentSortValue,
    sortOptions,
    searchTerm,
    isDebouncingSearch,
    setOpenJobRunDialog,
    handleJobSelect,
    handleSortChange,
    setCurrentPage,
    setCurrentOrderDir,
    setSearchTerm,
  };
};
