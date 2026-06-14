import { useCallback, useEffect, useState } from "react";

import {
  OrderDirection,
  type PaginationOptions,
} from "@flow/types/paginationOptions";

import useDebouncedSearch from "./useDebouncedSearch";

type PaginatedPage = { totalPages?: number | null };

type DataQuery = (
  workspaceId: string | undefined,
  keyword: string | undefined,
  paginationOptions: PaginationOptions,
) => {
  page?: PaginatedPage;
  refetch: () => Promise<unknown>;
  isFetching: boolean;
};

type Props<TQuery extends DataQuery, TOrderBy extends string> = {
  useDataQuery: TQuery;
  workspaceId?: string;
  defaultOrderBy: TOrderBy;
  defaultOrderDir?: OrderDirection;
};

export default function usePagination<
  TQuery extends DataQuery,
  TOrderBy extends string = string,
>({
  useDataQuery,
  workspaceId,
  defaultOrderBy,
  defaultOrderDir = OrderDirection.Desc,
}: Props<TQuery, TOrderBy>) {
  const [currentPage, setCurrentPage] = useState<number>(1);
  const [currentOrderBy, setCurrentOrderBy] =
    useState<TOrderBy>(defaultOrderBy);
  const [currentOrderDir, setCurrentOrderDir] =
    useState<OrderDirection>(defaultOrderDir);

  const { searchTerm, isDebouncingSearch, setSearchTerm } = useDebouncedSearch({
    initialSearchTerm: "",
    delay: 300,
    onDebounced: () => {
      refetch();
    },
  });

  const { page, refetch, isFetching } = useDataQuery(workspaceId, searchTerm, {
    page: currentPage,
    orderBy: currentOrderBy,
    orderDir: currentOrderDir,
  }) as {
    page: ReturnType<TQuery>["page"];
    refetch: () => Promise<unknown>;
    isFetching: boolean;
  };

  useEffect(() => {
    (async () => {
      await refetch();
    })();
  }, [currentPage, currentOrderDir, currentOrderBy, refetch]);

  const totalPages = page?.totalPages as number;

  const currentSortValue = `${currentOrderBy}_${currentOrderDir}`;

  const handleSortChange = useCallback((newSortValue: string) => {
    const [orderBy, orderDir] = newSortValue.split("_") as [
      TOrderBy,
      OrderDirection,
    ];
    setCurrentOrderBy(orderBy);
    setCurrentOrderDir(orderDir);
  }, []);

  return {
    page,
    totalPages,
    isFetching,
    currentPage,
    currentOrderBy,
    currentOrderDir,
    currentSortValue,
    searchTerm,
    isDebouncingSearch,
    setCurrentPage,
    setCurrentOrderBy,
    setCurrentOrderDir,
    setSearchTerm,
    handleSortChange,
  };
}
