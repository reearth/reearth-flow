import { useNavigate, useRouterState } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useState } from "react";

import { useDeployment } from "@flow/lib/gql";
import { useCurrentWorkspace } from "@flow/stores";
import { Deployment } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";
import { lastOfUrl as getDeploymentId } from "@flow/utils";

import { RouteOption } from "../WorkspaceLeftPanel";

export default () => {
  const navigate = useNavigate();

  const [openDeploymentAddDialog, setOpenDeploymentAddDialog] = useState(false);
  const [currentWorkspace] = useCurrentWorkspace();
  const [deploymentToBeEdited, setDeploymentToBeEdited] = useState<
    Deployment | undefined
  >(undefined);
  const [deploymentToBeDeleted, setDeploymentToBeDeleted] = useState<
    Deployment | undefined
  >(undefined);
  const [currentPage, setCurrentPage] = useState<number>(1);
  const [currentOrder, setCurrentOrder] = useState<OrderDirection>(
    OrderDirection.Desc,
  );
  const { useGetDeployments, useDeleteDeployment, executeDeployment } =
    useDeployment();

  const { page, refetch, isFetching } = useGetDeployments(
    currentWorkspace?.id,
    {
      page: currentPage,
      orderDir: currentOrder,
      orderBy: "updatedAt",
    },
  );

  useEffect(() => {
    (async () => {
      await refetch();
    })();
  }, [currentPage, currentOrder, refetch]);

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
    isFetching,
    currentPage,
    totalPages,
    currentOrder,
    setDeploymentToBeEdited,
    setOpenDeploymentAddDialog,
    setDeploymentToBeDeleted,
    handleDeploymentSelect,
    handleDeploymentDelete,
    handleDeploymentRun,
    setCurrentPage,
    setCurrentOrder,
  };
};

const getTab = (pathname: string): RouteOption =>
  pathname.includes("all") ? "all" : getDeploymentId(pathname);
