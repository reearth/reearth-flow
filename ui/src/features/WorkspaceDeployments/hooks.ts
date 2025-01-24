import { useNavigate, useRouter, useRouterState } from "@tanstack/react-router";
import { useCallback, useMemo, useRef, useState } from "react";

import { useDeployment } from "@flow/lib/gql";
import { useCurrentWorkspace } from "@flow/stores";
import { Deployment } from "@flow/types";
import { lastOfUrl as getDeploymentId } from "@flow/utils";

import usePagination from "../hooks/usePagination";
import { RouteOption } from "../WorkspaceLeftPanel";

const DEPLOYMENT_FETCH_RATE = 10;
export default () => {
  const ref = useRef<HTMLDivElement>(null);
  const navigate = useNavigate();
  const { history } = useRouter();

  const [openDeploymentAddDialog, setOpenDeploymentAddDialog] = useState(false);
  const [currentWorkspace] = useCurrentWorkspace();
  const [deploymentToBeEdited, setDeploymentToBeEdited] = useState<
    Deployment | undefined
  >(undefined);
  const [deploymentToBeDeleted, setDeploymentToBeDeleted] = useState<
    Deployment | undefined
  >(undefined);
  const [currentPage, setCurrentPage] = useState<number>(0);
  const { useGetDeploymentsInfinite, useDeleteDeployment, executeDeployment } =
    useDeployment();

  const { pages, hasNextPage, isFetching, fetchNextPage, isFetchingNextPage } =
    useGetDeploymentsInfinite(currentWorkspace?.id, DEPLOYMENT_FETCH_RATE);

  const {
    location: { pathname },
  } = useRouterState();

  const tab = getTab(pathname);

  const deployments: Deployment[] | undefined = useMemo(
    () => pages?.[currentPage]?.deployments,
    [pages, currentPage],
  );

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
      history.go(-1); // Go back to previous page
    },
    [
      currentWorkspace,
      deploymentToBeDeleted,
      deployments,
      history,
      useDeleteDeployment,
    ],
  );

  const handleDeploymentRun = useCallback(
    async (deployment?: Deployment) => {
      const d = deployment || selectedDeployment;
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
    [selectedDeployment, currentWorkspace, navigate, executeDeployment],
  );

  const { totalPages, handleNextPage, handlePrevPage, canGoNext } =
    usePagination<Deployment>(
      DEPLOYMENT_FETCH_RATE,
      hasNextPage,
      isFetchingNextPage,
      pages,
      fetchNextPage,
      currentPage,
      setCurrentPage,
    );

  return {
    ref,
    deployments,
    selectedDeployment,
    deploymentToBeDeleted,
    openDeploymentAddDialog,
    deploymentToBeEdited,
    setDeploymentToBeEdited,
    setOpenDeploymentAddDialog,
    setDeploymentToBeDeleted,
    handleDeploymentSelect,
    handleDeploymentDelete,
    handleDeploymentRun,
    totalPages,
    currentPage,
    hasNextPage: canGoNext,
    isFetching,
    isFetchingNextPage,
    handleNextPage,
    handlePrevPage,
  };
};

const getTab = (pathname: string): RouteOption =>
  pathname.includes("all") ? "all" : getDeploymentId(pathname);
