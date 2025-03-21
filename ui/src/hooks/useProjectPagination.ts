import { useEffect, useMemo, useState } from "react";

import { useProject } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { Workspace } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";

export default ({ workspace }: { workspace?: Workspace }) => {
  const t = useT();
  const { useGetWorkspaceProjects } = useProject();

  const [currentPage, setCurrentPage] = useState<number>(1);
  const [currentOrder, setCurrentOrder] = useState<OrderDirection>(
    OrderDirection.Desc,
  );

  const { page, refetch, isFetching } = useGetWorkspaceProjects(workspace?.id, {
    page: currentPage,
    orderDir: currentOrder,
    orderBy: "updatedAt",
  });

  const totalPages = useMemo(() => page?.totalPages as number, [page]);

  const projects = useMemo(() => page?.projects, [page]);

  const handleOrderChange = () => {
    setCurrentOrder?.(
      currentOrder === OrderDirection.Asc
        ? OrderDirection.Desc
        : OrderDirection.Asc,
    );
  };
  const orderDirections: Record<OrderDirection, string> = {
    DESC: t("Newest"),
    ASC: t("Oldest"),
  };

  useEffect(() => {
    (async () => {
      await refetch();
    })();
  }, [currentPage, currentOrder, refetch]);

  return {
    currentPage,
    projects,
    totalPages,
    isFetching,
    currentOrder,
    orderDirections,
    setCurrentPage,
    handleOrderChange,
  };
};
