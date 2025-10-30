import { useNavigate } from "@tanstack/react-router";
import { useCallback, useEffect, useRef, useState } from "react";

import { useDebouncedSearch } from "@flow/hooks";
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
  const [currentPage, setCurrentPage] = useState<number>(1);
  const [currentOrderBy, setCurrentOrderBy] = useState<JobOrderBy>(
    JobOrderBy.StartedAt,
  );
  const [currentOrderDir, setCurrentOrder] = useState<OrderDirection>(
    OrderDirection.Desc,
  );
  const { useGetJobs } = useJob();

  const { searchTerm, isDebouncingSearch, setSearchTerm } = useDebouncedSearch({
    initialSearchTerm: "",
    delay: 300,
    onDebounced: () => {
      refetch();
    },
  });

  const { page, refetch, isFetching } = useGetJobs(
    currentWorkspace?.id,
    searchTerm,
    {
      page: currentPage,
      orderBy: currentOrderBy,
      orderDir: currentOrderDir,
    },
  );

  const sortOptions = [
    {
      value: `${JobOrderBy.StartedAt}_${OrderDirection.Desc}`,
      label: t("Most Recent Started"),
    },
    {
      value: `${JobOrderBy.StartedAt}_${OrderDirection.Asc}`,
      label: t("Oldest Started"),
    },
    {
      value: `${JobOrderBy.CompletedAt}_${OrderDirection.Desc}`,
      label: t("Most Recent Completed"),
    },
    {
      value: `${JobOrderBy.CompletedAt}_${OrderDirection.Asc}`,
      label: t("Oldest Completed"),
    },
  ];

  const currentSortValue = `${currentOrderBy}_${currentOrderDir}`;

  useEffect(() => {
    (async () => {
      await refetch();
    })();
  }, [currentPage, currentOrderDir, currentOrderBy, refetch]);

  const totalPages = page?.totalPages as number;

  const jobs = page?.jobs;

  const handleJobSelect = useCallback(
    (job: Job) =>
      navigate({
        to: `/workspaces/${currentWorkspace?.id}/jobs/${job.id}`,
      }),
    [currentWorkspace, navigate],
  );

  const handleSortChange = useCallback((newSortValue: string) => {
    const [orderBy, orderDir] = newSortValue.split("_") as [
      JobOrderBy,
      OrderDirection,
    ];
    setCurrentOrderBy(orderBy);
    setCurrentOrder(orderDir);
  }, []);

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
    setCurrentOrder,
    setSearchTerm,
  };
};
