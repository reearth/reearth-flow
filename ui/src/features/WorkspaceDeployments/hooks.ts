import { useNavigate, useRouter, useRouterState } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { useDeployment } from "@flow/lib/gql";
import { useCurrentWorkspace } from "@flow/stores";
import { Deployment } from "@flow/types";
import { lastOfUrl as getDeploymentId } from "@flow/utils";

import { RouteOption } from "../WorkspaceLeftPanel";

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

  const { useGetDeploymentsInfinite, useDeleteDeployment, executeDeployment } =
    useDeployment();

  const { pages, hasNextPage, isFetching, fetchNextPage } =
    useGetDeploymentsInfinite(currentWorkspace?.id);

  const {
    location: { pathname },
  } = useRouterState();

  const tab = getTab(pathname);

  const deployments: Deployment[] | undefined = useMemo(
    () =>
      pages?.reduce((deployments, page) => {
        if (page?.deployments) {
          deployments.push(...page.deployments);
        }
        return deployments;
      }, [] as Deployment[]),
    [pages],
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

  // Auto fills the page
  useEffect(() => {
    if (
      ref.current &&
      ref.current?.scrollHeight <= document.documentElement.clientHeight &&
      hasNextPage &&
      !isFetching
    ) {
      fetchNextPage();
    }
  }, [isFetching, hasNextPage, ref, fetchNextPage]);

  // Loads more projects as scroll reaches the bottom
  useEffect(() => {
    const handleScroll = () => {
      if (
        window.innerHeight + document.documentElement.scrollTop + 5 >=
          document.documentElement.scrollHeight &&
        !isFetching &&
        hasNextPage
      ) {
        fetchNextPage();
      }
    };
    window.addEventListener("scroll", handleScroll);
    return () => window.removeEventListener("scroll", handleScroll);
  }, [isFetching, fetchNextPage, hasNextPage]);

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
  };
};

const getTab = (pathname: string): RouteOption =>
  pathname.includes("all") ? "all" : getDeploymentId(pathname);
