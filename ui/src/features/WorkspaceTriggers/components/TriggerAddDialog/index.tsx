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
import { DeploymentsDialog } from "@flow/features/WorkspaceDeployments/components/DeploymentsDialog";
import { useT } from "@flow/lib/i18n";

import { TriggerApiDrivenDetails } from "../TriggerApiDrivenDetail";
import TriggerProjectVariablesMappingDialog from "../TriggerWorkflowVariables";

import useHooks from "./hooks";

type Props = {
  setShowDialog: (show: boolean) => void;
};

const TriggerAddDialog: React.FC<Props> = ({ setShowDialog }) => {
  const t = useT();
  const {
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
    handleVariablesConfirm,
  } = useHooks({ setShowDialog });

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
            {pendingWorkflowData?.variables && (
              <DialogContentSection className="flex flex-col">
                <Label>{t("Workflow Variables")}</Label>
                <div
                  className="flex min-h-8 w-full cursor-pointer items-center rounded-md border bg-transparent px-3 py-1 text-sm"
                  onClick={() => setOpenTriggerProjectVariablesDialog(true)}>
                  <span className=" pr-2 whitespace-nowrap text-muted-foreground">
                    {t("Edit Variables")} (
                    {pendingWorkflowData.variables.length})
                  </span>
                </div>
              </DialogContentSection>
            )}
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
        <TriggerApiDrivenDetails
          createdTrigger={createdTrigger}
          onCopyToClipboard={handleCopyToClipboard}
        />
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
          onConfirm={handleVariablesConfirm}
          onCancel={() => setOpenTriggerProjectVariablesDialog(false)}
        />
      )}
    </Dialog>
  );
};

export { TriggerAddDialog };
