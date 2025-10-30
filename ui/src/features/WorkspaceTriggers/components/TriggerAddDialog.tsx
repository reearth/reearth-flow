import { useCallback, useState, useEffect } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  Label,
  DialogFooter,
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
  Input,
} from "@flow/components";
import { useDebouncedSearch } from "@flow/hooks";
import { useDeployment, useTrigger } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import { Deployment, DeploymentOrderBy, TimeInterval } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";

import { DeploymentsDialog } from "../../WorkspaceDeployments/components/DeploymentsDialog";

type Props = {
  setShowDialog: (show: boolean) => void;
};

const TriggerAddDialog: React.FC<Props> = ({ setShowDialog }) => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  const { createTrigger } = useTrigger();
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

  const handleSelectDeployment = (deployment: Deployment) => {
    const deploymentId = deployment.id;
    const selectedDeployment = deployments?.find((d) => d.id === deploymentId);
    setSelectedDeployment(selectedDeployment || null);
    setDeploymentId(deploymentId);
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

    await createTrigger(
      workspaceId,
      deploymentId,
      description,
      eventSource === "TIME_DRIVEN" ? timeInterval : undefined,
      eventSource === "API_DRIVEN" ? authToken : undefined,
    );

    setShowDialog(false);
  }, [
    currentWorkspace?.id,
    deploymentId,
    eventSource,
    authToken,
    timeInterval,
    setShowDialog,
    createTrigger,
    description,
  ]);

  return (
    <Dialog open={true} onOpenChange={() => setShowDialog(false)}>
      <DialogContent size="sm">
        <DialogTitle>{t("Create a new trigger")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection className="flex flex-col">
            <Label>{t("Description")}</Label>
            <Input
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder={t("Give your trigger a meaningful description...")}
            />
          </DialogContentSection>
          <DialogContentSection className="flex flex-col">
            <Label>{t("Deployment: ")}</Label>
            <div
              className="flex min-h-8 w-full items-center rounded-md border bg-transparent px-3 py-1 text-sm"
              onClick={() => setOpenSelectDeploymentsDialog(true)}>
              <span className="cursor-default pr-2 whitespace-nowrap text-muted-foreground">
                {t("Select Deployment: ")}
              </span>
              {selectedDeployment ? (
                <span className="cursor-default">
                  {selectedDeployment.description}@{selectedDeployment.version}
                </span>
              ) : (
                <span className="cursor-default">
                  {t("No Deployment Selected")}
                </span>
              )}
            </div>
          </DialogContentSection>
          <DialogContentSection className="flex-1">
            <Label htmlFor="event-source-selector">
              {t("Select Event Source")}
            </Label>
            <Select value={eventSource} onValueChange={handleSelectEventSource}>
              <SelectTrigger>
                <SelectValue placeholder={t("Select an event source")} />
              </SelectTrigger>
              <SelectContent>
                {Object.entries(eventSources).map(([value, label]) => (
                  <SelectItem key={value} value={value}>
                    {label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </DialogContentSection>
          {eventSource === "API_DRIVEN" && (
            <DialogContentSection className="flex flex-col">
              <Label>{t("Auth Token")}</Label>
              <Input
                value={authToken}
                onChange={(e) => setAuthToken(e.target.value)}
                placeholder={t("Add your auth token")}
              />
            </DialogContentSection>
          )}
          {eventSource === "TIME_DRIVEN" && (
            <DialogContentSection className="flex-1">
              <Label htmlFor="time-interval-selector">
                {t("Select Time Interval")}
              </Label>
              <Select
                value={timeInterval || "EVERY_DAY"} // Set default value here as well
                onValueChange={handleSelectTimeInterval}>
                <SelectTrigger>
                  <SelectValue placeholder={timeIntervals.EVERY_DAY} />
                </SelectTrigger>
                <SelectContent>
                  {Object.entries(timeIntervals).map(([value, label]) => (
                    <SelectItem key={value} value={value}>
                      {label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </DialogContentSection>
          )}
          <DialogContentSection>
            <p className="dark:font-light">
              {t("Are you sure you want to proceed?")}
            </p>
          </DialogContentSection>
        </DialogContentWrapper>
        <DialogFooter>
          <Button
            onClick={handleTriggerCreation}
            disabled={
              (eventSource === "API_DRIVEN" && !authToken) ||
              (eventSource === "TIME_DRIVEN" && !timeInterval) ||
              !eventSource ||
              !deploymentId ||
              !description
            }>
            {t("Add New Trigger")}
          </Button>
        </DialogFooter>
      </DialogContent>
      {openSelectDeploymentsDialog && (
        <DeploymentsDialog
          setShowDialog={() => setOpenSelectDeploymentsDialog(false)}
          deployments={deployments}
          onSelectDeployment={handleSelectDeployment}
          currentPage={currentPage}
          totalPages={totalPages}
          isFetching={isDebouncingSearch || isFetching}
          currentSortValue={currentSortValue}
          sortOptions={sortOptions}
          setSearchTerm={setSearchTerm}
          onSortChange={handleSortChange}
          setCurrentPage={setCurrentPage}
        />
      )}
    </Dialog>
  );
};

export { TriggerAddDialog };
