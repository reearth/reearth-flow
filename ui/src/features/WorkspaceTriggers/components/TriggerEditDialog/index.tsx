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
import { useT } from "@flow/lib/i18n";
import type { TimeInterval, Trigger } from "@flow/types";
import { getEventSourceLabels, getTimeIntervalLabels } from "@flow/utils";

import { TriggerNewAuthToken } from "../TriggerAuthToken";
import TriggerWorkflowVariablesMappingDialog from "../TriggerWorkflowVariables";
import { RequiredVariablesNotice } from "../TriggerWorkflowVariables/RequiredVariablesNotice";

import useHooks from "./hooks";

type Props = {
  selectedTrigger: Trigger;
  onDialogClose: () => void;
};

const TriggerEditDialog: React.FC<Props> = ({
  selectedTrigger,
  onDialogClose,
}) => {
  const t = useT();

  const {
    updatedEventSource,
    updatedTimeInterval,
    updatedDescription,
    variablesChanged,
    issuedAuthToken,
    handleEventSourceChange,
    handleTimeIntervalChange,
    handleDescriptionChange,
    handleTriggerUpdate,
    pendingWorkflowData,
    openTriggerWorkflowVariablesDialog,
    setOpenTriggerWorkflowVariablesDialog,
    handleVariablesConfirm,
    deploymentDefaultVariables,
    hasVariables,
    variableCount,
    hourlyDisabled,
    hourlyNotInPlan,
    definitions,
    missingRequired,
    isLoadingVariables,
    isUpdatingTrigger,
  } = useHooks({ selectedTrigger, onDialogClose });

  const eventSources = getEventSourceLabels(t);
  const timeIntervals = getTimeIntervalLabels(t);

  return (
    <Dialog open={true} onOpenChange={onDialogClose}>
      {issuedAuthToken ? (
        <TriggerNewAuthToken
          authToken={issuedAuthToken}
          onDone={onDialogClose}
        />
      ) : (
        <DialogContent size="sm">
          <DialogTitle>{t("Update Trigger")}</DialogTitle>
          <DialogContentWrapper>
            <DialogContentSection className="flex flex-col">
              <Label>{t("Description")}</Label>
              <Input
                value={updatedDescription}
                onChange={handleDescriptionChange}
                placeholder={t("Give your trigger a meaningful description...")}
              />
            </DialogContentSection>
            {hasVariables && (
              <DialogContentSection className="flex flex-col">
                <Label>{t("Workflow Variables")}</Label>
                <div
                  className="flex min-h-8 w-full cursor-pointer items-center rounded-md border bg-transparent px-3 py-1 text-sm aria-disabled:cursor-not-allowed aria-disabled:opacity-50"
                  aria-disabled={isLoadingVariables}
                  onClick={() =>
                    !isLoadingVariables &&
                    setOpenTriggerWorkflowVariablesDialog(true)
                  }>
                  <span className="pr-2 whitespace-nowrap text-muted-foreground">
                    {t("Edit Variables")} ({variableCount})
                  </span>
                </div>
                <RequiredVariablesNotice
                  missing={missingRequired}
                  eventSource={updatedEventSource}
                />
              </DialogContentSection>
            )}
            <DialogContentSection>
              <Label htmlFor="event-source-selector">
                {t("Select Event Source")}
              </Label>
              <Select
                value={updatedEventSource}
                onValueChange={(v) => v != null && handleEventSourceChange(v)}
                items={eventSources}>
                <SelectTrigger>
                  <SelectValue placeholder={eventSources[updatedEventSource]} />
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

            {updatedEventSource === "API_DRIVEN" &&
              selectedTrigger.eventSource !== "API_DRIVEN" && (
                <DialogContentSection>
                  <p className="text-xs text-muted-foreground">
                    {t(
                      "An auth token is created for the trigger and shown once after you save.",
                    )}
                  </p>
                </DialogContentSection>
              )}
            {updatedEventSource === "TIME_DRIVEN" && (
              <DialogContentSection>
                <Label htmlFor="time-interval-selector">
                  {t("Select Time Interval")}
                </Label>
                <Select
                  value={updatedTimeInterval || "EVERY_DAY"}
                  onValueChange={(value) =>
                    value != null &&
                    handleTimeIntervalChange(value.toString() as TimeInterval)
                  }
                  items={timeIntervals}>
                  <SelectTrigger>
                    <SelectValue placeholder={timeIntervals.EVERY_DAY} />
                  </SelectTrigger>
                  <SelectContent>
                    {Object.entries(timeIntervals).map(([value, label]) => (
                      <SelectItem
                        key={value}
                        value={value}
                        disabled={value === "EVERY_HOUR" && hourlyDisabled}>
                        {label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                {hourlyNotInPlan && (
                  <p className="text-xs text-muted-foreground">
                    {t(
                      "Hourly schedules aren't included in this workspace's plan.",
                    )}
                  </p>
                )}
              </DialogContentSection>
            )}
          </DialogContentWrapper>
          <DialogFooter>
            <Button
              onClick={handleTriggerUpdate}
              disabled={
                isUpdatingTrigger ||
                (updatedEventSource === "TIME_DRIVEN" &&
                  missingRequired.length > 0) ||
                (updatedEventSource === selectedTrigger.eventSource &&
                  updatedTimeInterval === selectedTrigger.timeInterval &&
                  (updatedDescription === selectedTrigger.description ||
                    !updatedDescription.trim()) &&
                  !variablesChanged)
              }>
              {isUpdatingTrigger ? t("Updating...") : t("Update Trigger")}
            </Button>
          </DialogFooter>
        </DialogContent>
      )}
      {pendingWorkflowData?.variables && (
        <TriggerWorkflowVariablesMappingDialog
          isOpen={openTriggerWorkflowVariablesDialog}
          onOpenChange={setOpenTriggerWorkflowVariablesDialog}
          variables={pendingWorkflowData.variables}
          deploymentDefaults={deploymentDefaultVariables}
          definitions={definitions}
          workflowName={selectedTrigger.deployment.projectName || ""}
          onConfirm={handleVariablesConfirm}
          onCancel={() => setOpenTriggerWorkflowVariablesDialog(false)}
        />
      )}
    </Dialog>
  );
};

export { TriggerEditDialog };
