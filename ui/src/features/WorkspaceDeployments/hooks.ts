import { useNavigate, useRouterState } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useState } from "react";

import { useDebouncedSearch } from "@flow/hooks";
import { useDeployment } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import { Deployment, DeploymentOrderBy } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";
import { lastOfUrl as getDeploymentId } from "@flow/utils";

import { RouteOption } from "../WorkspaceLeftPanel";

export default () => {
  const navigate = useNavigate();
  const t = useT();

  const [openDeploymentAddDialog, setOpenDeploymentAddDialog] = useState(false);
  const [currentWorkspace] = useCurrentWorkspace();
  const [deploymentToBeEdited, setDeploymentToBeEdited] = useState<
    Deployment | undefined
  >(undefined);
  const [deploymentToBeDeleted, setDeploymentToBeDeleted] = useState<
    Deployment | undefined
  >(undefined);
  const [currentPage, setCurrentPage] = useState<number>(1);
  const [currentOrderBy, setCurrentOrderBy] = useState<DeploymentOrderBy>(
    DeploymentOrderBy.UpdatedAt,
  );
  const [currentOrderDir, setCurrentOrderDir] = useState<OrderDirection>(
    OrderDirection.Desc,
  );
  const { useGetDeployments, useDeleteDeployment, executeDeployment } =
    useDeployment();

  const { searchTerm, isDebouncingSearch, setSearchTerm } = useDebouncedSearch({
    initialSearchTerm: "",
    delay: 300,
    onDebounced: () => {
      refetch();
    },
  });

  const { page, refetch, isFetching } = useGetDeployments(
    currentWorkspace?.id,
    searchTerm,
    {
      page: currentPage,
      orderDir: currentOrderDir,
      orderBy: currentOrderBy,
    },
  );

  const sortOptions = [
    {
      value: `${DeploymentOrderBy.UpdatedAt}_${OrderDirection.Desc}`,
      label: t("Latest Updated"),
    },
    {
      value: `${DeploymentOrderBy.UpdatedAt}_${OrderDirection.Asc}`,
      label: t("Oldest Updated"),
    },
    {
      value: `${DeploymentOrderBy.Version}_${OrderDirection.Desc}`,
      label: t("Latest Version"),
    },
    {
      value: `${DeploymentOrderBy.Version}_${OrderDirection.Asc}`,
      label: t("Oldest Version"),
    },
    {
      value: `${DeploymentOrderBy.Description}_${OrderDirection.Asc}`,
      label: t("A To Z"),
    },
    {
      value: `${DeploymentOrderBy.Description}_${OrderDirection.Desc}`,
      label: t("Z To A"),
    },
  ];

  useEffect(() => {
    (async () => {
      await refetch();
    })();
  }, [currentPage, currentOrderDir, currentOrderBy, refetch]);

  const currentSortValue = `${currentOrderBy}_${currentOrderDir}`;

  const handleSortChange = useCallback((newSortValue: string) => {
    const [orderBy, orderDir] = newSortValue.split("_") as [
      DeploymentOrderBy,
      OrderDirection,
    ];
    setCurrentOrderBy(orderBy);
    setCurrentOrderDir(orderDir);
  }, []);

  const totalPages = page?.totalPages as number;

  const {
    location: { pathname },
  } = useRouterState();

  const tab = getTab(pathname);

  const deployments = page?.deployments;

  const selectedDeployment = useMemo(
    () => deployments?.find((deployment) => deployment.id === tab),
    [tab, deployments],
  );

  const handleDeploymentSelect = useCallback(
    (deployment: Deployment) =>
      navigate({
        to: `/workspaces/${currentWorkspace?.id}/deployments/${deployment.id}`,
      }),
    [currentWorkspace, navigate],
  );

  const handleDeploymentDelete = useCallback(
    async (deployment?: Deployment) => {
      const d =
        deployment ||
        deployments?.find((d2) => d2.id === deploymentToBeDeleted?.id);
      if (!d || !currentWorkspace) return;
      await useDeleteDeployment(d.id, currentWorkspace.id);
      setDeploymentToBeDeleted(undefined);
      navigate({
        to: `/workspaces/${currentWorkspace.id}/deployments`,
      });
    },
    [
      currentWorkspace,
      deploymentToBeDeleted,
      deployments,
      navigate,
      useDeleteDeployment,
    ],
  );

  const handleDeploymentRun = useCallback(
    async (deployment?: Deployment) => {
      const d = deployment;
      if (!d || !currentWorkspace) return;
      const jobData = await executeDeployment({
        deploymentId: d.id,
      });
      if (jobData) {
        navigate({
          to: `/workspaces/${currentWorkspace.id}/jobs/${jobData.job?.id}`,
        });
      }
    },
    [currentWorkspace, navigate, executeDeployment],
  );

  return {
    deployments,
    selectedDeployment,
    deploymentToBeDeleted,
    openDeploymentAddDialog,
    deploymentToBeEdited,
    isDebouncingSearch,
    isFetching,
    currentPage,
    currentSortValue,
    sortOptions,
    totalPages,
    handleDeploymentSelect,
    handleDeploymentDelete,
    handleDeploymentRun,
    handleSortChange,
    setCurrentPage,
    setCurrentOrderDir,
    setDeploymentToBeDeleted,
    setDeploymentToBeEdited,
    setOpenDeploymentAddDialog,

    setSearchTerm,
  };
};

const getTab = (pathname: string): RouteOption =>
  pathname.includes("all") ? "all" : getDeploymentId(pathname);
