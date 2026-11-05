import { useNavigate, useRouterState } from "@tanstack/react-router";
import { useCallback, useMemo, useState } from "react";

import { usePagination } from "@flow/hooks";
import { useDeployment } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import type { AnyWorkflowVariable, Deployment } from "@flow/types";
import { DeploymentOrderBy } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";
import { lastOfUrl as getDeploymentId } from "@flow/utils";

import type { RouteOption } from "../WorkspaceLeftPanel";

import { toRunOverrides } from "./runOverrides";

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
  const [deploymentToBeRun, setDeploymentToBeRun] = useState<
    Deployment | undefined
  >(undefined);
  const { useGetDeployments, useDeleteDeployment, executeDeployment } =
    useDeployment();

  const {
    page,
    totalPages,
    isFetching,
    currentPage,
    currentSortValue,
    isDebouncingSearch,
    setCurrentPage,
    setCurrentOrderDir,
    setSearchTerm,
    handleSortChange,
  } = usePagination({
    useDataQuery: useGetDeployments,
    workspaceId: currentWorkspace?.id,
    defaultOrderBy: DeploymentOrderBy.UpdatedAt,
  });

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

  const handleDeploymentRun = useCallback((deployment?: Deployment) => {
    if (deployment) {
      setDeploymentToBeRun(deployment);
    }
  }, []);

  // A deployment with variables opens the variables dialog instead of the
  // plain confirmation, so its values can be changed for the run.
  const runNeedsVariables = !!deploymentToBeRun?.parameters.length;

  const handleDeploymentRunConfirmed = useCallback(
    // Reports whether the run started, so the variables dialog can stay open
    // with its edits when it did not.
    async (
      deployment?: Deployment,
      variables?: AnyWorkflowVariable[],
    ): Promise<boolean> => {
      if (!currentWorkspace || !deployment) return false;
      const jobData = await executeDeployment({
        deploymentId: deployment.id,
        variables: variables
          ? toRunOverrides(deployment.parameters, variables)
          : undefined,
      });
      // A refused run returns no job, and its toast explains why.
      const jobId = jobData.job?.id;
      if (!jobId) return false;
      navigate({
        to: `/workspaces/${currentWorkspace.id}/jobs/${jobId}`,
      });
      return true;
    },
    [currentWorkspace, navigate, executeDeployment],
  );

  return {
    deployments,
    selectedDeployment,
    deploymentToBeDeleted,
    deploymentToBeRun,
    runNeedsVariables,
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
    handleDeploymentRunConfirmed,
    handleSortChange,
    setCurrentPage,
    setCurrentOrderDir,
    setDeploymentToBeDeleted,
    setDeploymentToBeRun,
    setDeploymentToBeEdited,
    setOpenDeploymentAddDialog,
    setSearchTerm,
  };
};

const getTab = (pathname: string): RouteOption =>
  pathname.includes("all") ? "all" : getDeploymentId(pathname);
