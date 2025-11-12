import { useCallback, useEffect, useState } from "react";

import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useDebouncedSearch } from "@flow/hooks";
import { useTrigger, useDeployment } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import {
  Trigger,
  TimeInterval,
  Deployment,
  DeploymentOrderBy,
} from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";
import { copyToClipboard } from "@flow/utils/copyToClipboard";

import { useDeploymentWorkflowVariables } from "../TriggerWorkflowVariables/useDeploymentWorkflowVariables";

export default ({
  setShowDialog,
}: {
  setShowDialog: (show: boolean) => void;
}) => {
  const t = useT();
  const { toast } = useToast();

  const [currentWorkspace] = useCurrentWorkspace();
  const { createTrigger } = useTrigger();
  const [createdTrigger, setCreatedTrigger] = useState<Trigger | undefined>(
    undefined,
  );
  const [deploymentId, setDeploymentId] = useState<string>("");
  const [selectedDeployment, setSelectedDeployment] =
    useState<Deployment | null>(null);
  const [eventSource, setEventSource] = useState<string>("");
  const [timeInterval, setTimeInterval] = useState<TimeInterval | undefined>(
    undefined,
  );
  const [authToken, setAuthToken] = useState<string>("");
  const [description, setDescription] = useState<string>("");
  const [currentPage, setCurrentPage] = useState<number>(1);
  const [currentOrderBy, setCurrentOrderBy] = useState<DeploymentOrderBy>(
    DeploymentOrderBy.UpdatedAt,
  );
  const [currentOrderDir, setCurrentOrderDir] = useState<OrderDirection>(
    OrderDirection.Desc,
  );
  const { useGetDeployments } = useDeployment();

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

  const deployments = page?.deployments;
  const totalPages = page?.totalPages as number;
  const [openSelectDeploymentsDialog, setOpenSelectDeploymentsDialog] =
    useState<boolean>(false);

  useEffect(() => {
    if (eventSource === "API_DRIVEN") {
      setTimeInterval(undefined);
    } else {
      setAuthToken("");
    }
  }, [eventSource]);

  const {
    pendingWorkflowData,
    workflowVariablesObject,
    openTriggerProjectVariablesDialog,
    setOpenTriggerProjectVariablesDialog,
    handleWorkflowFetch,
    handleVariablesConfirm,
    getVariablesToSave,
  } = useDeploymentWorkflowVariables();

  useEffect(() => {
    if (
      pendingWorkflowData &&
      pendingWorkflowData.variables &&
      pendingWorkflowData.variables.length > 0
    ) {
      setOpenTriggerProjectVariablesDialog(true);
    }
  }, [pendingWorkflowData, setOpenTriggerProjectVariablesDialog]);

  const handleSelectDeployment = (deployment: Deployment) => {
    const deploymentId = deployment.id;
    const selectedDeployment = deployments?.find((d) => d.id === deploymentId);
    setSelectedDeployment(selectedDeployment || null);
    setDeploymentId(deploymentId);
    handleWorkflowFetch(selectedDeployment?.workflowUrl);
    setOpenTriggerProjectVariablesDialog(true);
  };

  const handleSelectEventSource = (eventSource: string) => {
    setEventSource(eventSource);
    if (eventSource === "TIME_DRIVEN") {
      setTimeInterval("EVERY_DAY");
    } else {
      setTimeInterval(undefined);
    }
  };

  const eventSources: Record<string, string> = {
    API_DRIVEN: t("API Driven"),
    TIME_DRIVEN: t("Time Driven"),
  };

  const handleSelectTimeInterval = (timeInterval: TimeInterval) => {
    setTimeInterval(timeInterval);
  };

  const timeIntervals: Record<TimeInterval, string> = {
    EVERY_DAY: t("Every Day"),
    EVERY_HOUR: t("Every Hour"),
    EVERY_WEEK: t("Every Week"),
    EVERY_MONTH: t("Every Month"),
  };

  const handleTriggerCreation = useCallback(async () => {
    const workspaceId = currentWorkspace?.id;

    if (!workspaceId) {
      console.error("No workspace ID found");
      return;
    }

    // Only save variables if they differ from deployment defaults
    const variablesToSave = getVariablesToSave();

    const { trigger: createdTrigger } = await createTrigger(
      workspaceId,
      deploymentId,
      description,
      eventSource === "TIME_DRIVEN" ? timeInterval : undefined,
      eventSource === "API_DRIVEN" ? authToken : undefined,
      variablesToSave,
    );

    setCreatedTrigger(createdTrigger);

    if (eventSource === "TIME_DRIVEN") {
      setShowDialog(false);
    }
  }, [
    currentWorkspace?.id,
    deploymentId,
    eventSource,
    authToken,
    timeInterval,
    setShowDialog,
    createTrigger,
    description,
    getVariablesToSave,
  ]);

  const handleCopyToClipboard = useCallback(
    (url: string) => {
      copyToClipboard(url);
      toast({
        title: t("Copied to clipboard"),
        description: t("URL copied to clipboard"),
      });
    },
    [t, toast],
  );

  return {
    createdTrigger,
    eventSources,
    eventSource,
    timeIntervals,
    timeInterval,
    authToken,
    description,
    deployments,
    deploymentId,
    isFetching,
    isDebouncingSearch,
    totalPages,
    currentPage,
    currentSortValue,
    sortOptions,
    openSelectDeploymentsDialog,
    selectedDeployment,
    setSearchTerm,
    setDescription,
    setOpenSelectDeploymentsDialog,
    setAuthToken,
    setShowDialog,
    setTimeInterval,
    handleCopyToClipboard,
    handleSelectDeployment,
    handleSelectEventSource,
    handleSelectTimeInterval,
    handleTriggerCreation,
    setCurrentPage,
    handleSortChange,
    // Project Params for Workflow Variables
    handleVariablesConfirm,
    pendingWorkflowData,
    workflowVariablesObject,
    openTriggerProjectVariablesDialog,
    setOpenTriggerProjectVariablesDialog,
  };
};
