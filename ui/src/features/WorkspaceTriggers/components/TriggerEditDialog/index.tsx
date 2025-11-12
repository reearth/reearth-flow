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
import { TimeInterval, Trigger } from "@flow/types";

import TriggerProjectVariablesMappingDialog from "../TriggerWorkflowVariables";

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
    updatedAuthToken,
    updatedTimeInterval,
    updatedDescription,
    variablesChanged,
    handleEventSourceChange,
    handleAuthTokenChange,
    handleTimeIntervalChange,
    handleDescriptionChange,
    handleTriggerUpdate,
    pendingWorkflowData,
    openTriggerProjectVariablesDialog,
    setOpenTriggerProjectVariablesDialog,
    handleVariablesConfirm,
    hasVariables,
    variableCount,
  } = useHooks({ selectedTrigger, onDialogClose });

  const eventSources: Record<string, string> = {
    API_DRIVEN: t("API Driven"),
    TIME_DRIVEN: t("Time Driven"),
  };

  const timeIntervals: Record<TimeInterval, string> = {
    EVERY_DAY: t("Every Day"),
    EVERY_HOUR: t("Every Hour"),
    EVERY_WEEK: t("Every Week"),
    EVERY_MONTH: t("Every Month"),
  };

  return (
    <Dialog open={true} onOpenChange={onDialogClose}>
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
                className="flex min-h-8 w-full cursor-pointer items-center rounded-md border bg-transparent px-3 py-1 text-sm"
                onClick={() => setOpenTriggerProjectVariablesDialog(true)}>
                <span className="pr-2 whitespace-nowrap text-muted-foreground">
                  {t("Edit Variables")} ({variableCount})
                </span>
              </div>
            </DialogContentSection>
          )}
          <DialogContentSection className="flex-1">
            <Label htmlFor="event-source-selector">
              {t("Select Event Source")}
            </Label>
            <Select
              value={updatedEventSource}
              onValueChange={handleEventSourceChange}>
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

          {updatedEventSource === "API_DRIVEN" && (
            <DialogContentSection className="flex flex-col">
              <Label>{t("Auth Token")}</Label>
              <Input
                value={updatedAuthToken}
                onChange={handleAuthTokenChange}
                placeholder={t("Add your auth token")}
              />
            </DialogContentSection>
          )}
          {updatedEventSource === "TIME_DRIVEN" && (
            <DialogContentSection className="flex-1">
              <Label htmlFor="time-interval-selector">
                {t("Select Time Interval")}
              </Label>
              <Select
                value={updatedTimeInterval || "EVERY_DAY"}
                onValueChange={(value) =>
                  handleTimeIntervalChange(value.toString() as TimeInterval)
                }>
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
        </DialogContentWrapper>
        <DialogFooter>
          <Button
            onClick={handleTriggerUpdate}
            disabled={
              updatedEventSource === selectedTrigger.eventSource &&
              updatedTimeInterval === selectedTrigger.timeInterval &&
              updatedAuthToken === selectedTrigger.authToken &&
              (updatedDescription === selectedTrigger.description ||
                !updatedDescription.trim()) &&
              !variablesChanged
            }>
            {t("Update Trigger")}
          </Button>
        </DialogFooter>
      </DialogContent>
      {pendingWorkflowData?.variables && (
        <TriggerProjectVariablesMappingDialog
          isOpen={openTriggerProjectVariablesDialog}
          onOpenChange={setOpenTriggerProjectVariablesDialog}
          variables={pendingWorkflowData.variables}
          workflowName={selectedTrigger.deployment.projectName || ""}
          onConfirm={handleVariablesConfirm}
          onCancel={() => setOpenTriggerProjectVariablesDialog(false)}
        />
      )}
    </Dialog>
  );
};

export { TriggerEditDialog };
