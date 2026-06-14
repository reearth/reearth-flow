import { useMemo } from "react";

import { useProject } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { ProjectOrderBy, Workspace } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";

import usePagination from "./usePagination";

export default ({ workspace }: { workspace?: Workspace }) => {
  const t = useT();
  const { useGetWorkspaceProjects } = useProject();

  const {
    page,
    totalPages,
    isFetching,
    currentPage,
    currentOrderBy,
    currentSortValue,
    searchTerm,
    isDebouncingSearch,
    setCurrentPage,
    setSearchTerm,
    handleSortChange,
  } = usePagination({
    useDataQuery: useGetWorkspaceProjects,
    workspaceId: workspace?.id,
    defaultOrderBy: ProjectOrderBy.CreatedAt,
  });

  const projects = useMemo(() => page?.projects, [page]);

  const sortOptions = [
    {
      value: `${ProjectOrderBy.CreatedAt}_${OrderDirection.Desc}`,
      label: t("Last Created"),
    },
    {
      value: `${ProjectOrderBy.CreatedAt}_${OrderDirection.Asc}`,
      label: t("First Created"),
    },
    {
      value: `${ProjectOrderBy.Name}_${OrderDirection.Asc}`,
      label: t("A To Z"),
    },
    {
      value: `${ProjectOrderBy.Name}_${OrderDirection.Desc}`,
      label: t("Z To A"),
    },
    {
      value: `${ProjectOrderBy.UpdatedAt}_${OrderDirection.Desc}`,
      label: t("Latest Updated"),
    },
    {
      value: `${ProjectOrderBy.UpdatedAt}_${OrderDirection.Asc}`,
      label: t("Oldest Updated"),
    },
  ];

  const orderDirections: Record<OrderDirection, string> = {
    DESC: t("Newest"),
    ASC: t("Oldest"),
  };

  return {
    currentPage,
    projects,
    totalPages,
    isFetching,
    currentOrderBy,
    currentSortValue,
    searchTerm,
    orderDirections,
    sortOptions,
    isDebouncingSearch,
    setCurrentPage,
    setSearchTerm,
    handleSortChange,
  };
};
