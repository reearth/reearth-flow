import { CopyIcon, QuestionIcon } from "@phosphor-icons/react";

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
  IconButton,
  Tooltip,
  TooltipTrigger,
  TooltipContent,
} from "@flow/components";
import { DeploymentsDialog } from "@flow/features/WorkspaceDeployments/components/DeploymentsDialog";
import { useT } from "@flow/lib/i18n";

import TriggerProjectVariablesMappingDialog from "../TriggerWorkflowVariables";

import useHooks from "./hooks";

type Props = {
  setShowDialog: (show: boolean) => void;
};

const TriggerAddDialog: React.FC<Props> = ({ setShowDialog }) => {
  const t = useT();
  const {
    apiUrl,
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
    handleCopyToClipboard,
    handleSelectDeployment,
    handleSelectEventSource,
    handleSelectTimeInterval,
    handleTriggerCreation,
    setCurrentPage,
    handleSortChange,
    pendingWorkflowData,
    openTriggerProjectVariablesDialog,
    setOpenTriggerProjectVariablesDialog,
  } = useHooks({ setShowDialog });

  console.log("SELECTED DEPLOYMENT: ", selectedDeployment);
  console.log("PENDING WORKFLOW DATA: ", pendingWorkflowData?.variables);
  return (
    <Dialog open={true} onOpenChange={() => setShowDialog(false)}>
      {!createdTrigger && (
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
                    {selectedDeployment.description}@
                    {selectedDeployment.version}
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
              <Select
                value={eventSource}
                onValueChange={handleSelectEventSource}>
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
      )}
      {createdTrigger?.eventSource === "API_DRIVEN" && (
        <DialogContent size="sm">
          <DialogTitle>{t("How to Trigger API Driven Event:")}</DialogTitle>
          <DialogContentWrapper className="overflow-hidden">
            <div className="space-y-3 text-sm text-muted-foreground">
              <div className="flex flex-nowrap items-center gap-2">
                <span className="font-semibold">1. {t("Endpoint:")}</span>
                <div className="max-w-[200px] overflow-x-auto overflow-y-hidden p-1">
                  <span className="rounded border bg-background px-2 py-1 font-mono text-xs whitespace-nowrap">
                    POST {apiUrl}/api/triggers/{createdTrigger.id}/run
                  </span>
                </div>
                <IconButton
                  size="icon"
                  variant="ghost"
                  className="ml-1 shrink-0"
                  onClick={() =>
                    handleCopyToClipboard(
                      `${apiUrl}/api/triggers/${createdTrigger.id}/run`,
                    )
                  }
                  icon={<CopyIcon />}
                />
              </div>
              <div>
                <span className="font-semibold">2. {t("Auth:")}</span>{" "}
                {t(
                  `Add token to "Authorization: Bearer ${createdTrigger.authToken}" header`,
                )}
              </div>
              <div className="flex flex-wrap items-center">
                <span className="font-semibold">
                  3. {t("Custom Variables")}
                </span>

                <Tooltip>
                  <TooltipTrigger asChild>
                    <div className="cursor-pointer p-1">
                      <QuestionIcon className="h-4 w-4" weight="thin" />
                    </div>
                  </TooltipTrigger>
                  <TooltipContent side="top" align="end" className="bg-primary">
                    <div className="max-w-[300px] text-xs text-muted-foreground">
                      {t(
                        'Pass {"with": {"key": "value"}} in body to inject dynamic parameters into workflow execution. These variables override/supplement default workflow values and are accessible in nodes.',
                      )}
                    </div>
                  </TooltipContent>
                </Tooltip>

                <span className="mx-1">:</span>

                <span className="text-muted-foreground">
                  {t('Pass {"with": {"key": "value"}} in body')}
                </span>
              </div>

              <div>
                <span className="font-semibold">4. {t("Callback:")}</span>{" "}
                {t('Optional "notificationUrl" for status updates')}
              </div>
              <div>
                <span className="font-semibold">5. {t("Response:")}</span>{" "}
                {t("Returns runId, deploymentId, and job status")}
              </div>
            </div>

            <p className="mt-2 border-t border-muted-foreground/20 pt-2 text-xs">
              {t(
                "You can review these details any time on the trigger's details page",
              )}
            </p>
          </DialogContentWrapper>
        </DialogContent>
      )}
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
      {pendingWorkflowData?.variables && (
        <TriggerProjectVariablesMappingDialog
          isOpen={openTriggerProjectVariablesDialog}
          onOpenChange={setOpenTriggerProjectVariablesDialog}
          variables={pendingWorkflowData?.variables || []}
          workflowName={pendingWorkflowData?.workflowName || ""}
          onConfirm={() => setOpenTriggerProjectVariablesDialog(false)}
          onCancel={() => setOpenTriggerProjectVariablesDialog(false)}
        />
      )}
    </Dialog>
  );
};

export { TriggerAddDialog };
