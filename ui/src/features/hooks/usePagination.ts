import { useCallback, useMemo, Dispatch, SetStateAction } from "react";

import type { Deployment, Job, Project, Trigger } from "@flow/types";

type PageData<T> =
  | {
      triggers?: T[];
      jobs?: T[];
      deployments?: T[];
      projects?: T[];
      totalCount: number;
    }
  | undefined;

export default <T extends Trigger | Job | Deployment | Project>(
  fetchRate: number,
  hasNextPage: boolean,
  isFetchingNextPage: boolean,
  pages: PageData<T>[] | undefined,
  fetchNextPage: () => Promise<any>,
  currentPage: number,
  setCurrentPage: Dispatch<SetStateAction<number>>,
) => {
  const totalCount: number = pages?.[0]?.totalCount ?? 0;
  const totalPages = Math.ceil(totalCount / fetchRate);

  const canGoNext = useMemo(() => {
    if (hasNextPage) return true;
    return currentPage < totalPages - 1;
  }, [currentPage, totalPages, hasNextPage]);

  const handleNextPage = useCallback(() => {
    if (!isFetchingNextPage && canGoNext) {
      const nextPageData = pages?.[currentPage + 1];
      const hasData =
        nextPageData?.triggers ||
        nextPageData?.jobs ||
        nextPageData?.deployments ||
        nextPageData?.projects;

      if (hasData) {
        setCurrentPage((prev) => prev + 1);
      } else if (hasNextPage) {
        fetchNextPage().then(() => {
          setCurrentPage((prev) => prev + 1);
        });
      }
    }
  }, [
    canGoNext,
    isFetchingNextPage,
    pages,
    currentPage,
    hasNextPage,
    fetchNextPage,
    setCurrentPage,
  ]);

  const handlePrevPage = useCallback(() => {
    if (currentPage > 0) {
      setCurrentPage((prev) => prev - 1);
    }
  }, [currentPage, setCurrentPage]);

  return {
    totalPages,
    currentPage,
    hasNextPage: canGoNext,
    isFetchingNextPage,
    handleNextPage,
    handlePrevPage,
    canGoNext,
  };
};
