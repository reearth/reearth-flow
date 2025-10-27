import { useCallback, useEffect, useMemo, useState } from "react";

import { useProject } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { ProjectOrderBy, Workspace } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";

import useDebouncedSearch from "./useDebouncedSearch";

export default ({ workspace }: { workspace?: Workspace }) => {
  const t = useT();
  const { useGetWorkspaceProjects } = useProject();

  const [currentPage, setCurrentPage] = useState<number>(1);
  const [currentOrderBy, setCurrentOrderBy] = useState<ProjectOrderBy>(
    ProjectOrderBy.CreatedAt,
  );
  const [currentOrderDir, setCurrentOrder] = useState<OrderDirection>(
    OrderDirection.Desc,
  );

  const { searchTerm, isDebouncingSearch, setSearchTerm } = useDebouncedSearch({
    initialSearchTerm: "",
    delay: 300,
    onDebounced: () => {
      refetch();
    },
  });

  const { page, refetch, isFetching } = useGetWorkspaceProjects(
    workspace?.id,
    searchTerm,
    {
      page: currentPage,
      orderDir: currentOrderDir,
      orderBy: "updatedAt",
    },
  );

  const totalPages = useMemo(() => page?.totalPages as number, [page]);

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

  const currentSortValue = `${currentOrderBy}_${currentOrderDir}`;

  useEffect(() => {
    (async () => {
      await refetch();
    })();
  }, [currentPage, currentOrderDir, currentOrderBy, refetch]);

  const handleSortChange = useCallback((newSortValue: string) => {
    const [orderBy, orderDir] = newSortValue.split("_") as [
      ProjectOrderBy,
      OrderDirection,
    ];
    setCurrentOrderBy(orderBy);
    setCurrentOrder(orderDir);
  }, []);
  const orderDirections: Record<OrderDirection, string> = {
    DESC: t("Newest"),
    ASC: t("Oldest"),
  };

  useEffect(() => {
    (async () => {
      await refetch();
    })();
  }, [currentPage, currentOrderDir, currentOrderBy, refetch]);

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
