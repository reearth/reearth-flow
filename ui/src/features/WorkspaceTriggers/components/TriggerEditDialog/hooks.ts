import { isEqual } from "lodash-es";
import type { ChangeEvent } from "react";
import { useCallback, useEffect, useRef, useState } from "react";

import { useTrigger } from "@flow/lib/gql";
import type { Trigger, TimeInterval, EventSourceType } from "@flow/types";

import { useHourlySchedule } from "../../useHourlySchedule";
import { useTriggerWorkflowVariables } from "../TriggerWorkflowVariables/useTriggerWorkflowVariables";

export default ({
  selectedTrigger,
  onDialogClose,
}: {
  selectedTrigger: Trigger;
  onDialogClose: () => void;
}) => {
  const { useUpdateTrigger } = useTrigger();
  const [updatedDescription, setUpdatedDescription] = useState(
    selectedTrigger.description || "",
  );

  const [updatedEventSource, setUpdatedEventSource] = useState(
    selectedTrigger.eventSource,
  );
  // A token issued by this update, shown once before the dialog closes.
  const [issuedAuthToken, setIssuedAuthToken] = useState<string | undefined>(
    undefined,
  );
  const [updatedTimeInterval, setUpdatedTimeInterval] = useState<
    TimeInterval | undefined
  >(selectedTrigger.timeInterval || undefined);

  const handleEventSourceChange = (eventSource: EventSourceType) => {
    setUpdatedEventSource(eventSource);
    if (eventSource === "TIME_DRIVEN") {
      setUpdatedTimeInterval(selectedTrigger.timeInterval || "EVERY_DAY");
    } else {
      setUpdatedTimeInterval(undefined);
    }
  };

  const handleTimeIntervalChange = useCallback((timeInterval: TimeInterval) => {
    setUpdatedTimeInterval(timeInterval);
  }, []);

  const handleDescriptionChange = useCallback(
    (e: ChangeEvent<HTMLInputElement>) => {
      setUpdatedDescription(e.target.value);
    },
    [],
  );

  const {
    workflowVariablesObject,
    pendingWorkflowData,
    openTriggerWorkflowVariablesDialog,
    setOpenTriggerWorkflowVariablesDialog,
    handleVariablesConfirm,
    getVariablesToSave,
    loadDeploymentVariables,
    isLoadingVariables,
    deploymentDefaultVariables,
    definitions,
    missingRequired,
  } = useTriggerWorkflowVariables(selectedTrigger.variables);

  // Load the deployment's variables to compare the trigger's values against
  useEffect(() => {
    // Pass true if trigger has custom variables (don't overwrite them)
    const hasCustomVariables = !!selectedTrigger.variables;
    loadDeploymentVariables(selectedTrigger.deployment, hasCustomVariables);
  }, [
    selectedTrigger.deployment,
    selectedTrigger.variables,
    loadDeploymentVariables,
  ]);

  // The ref blocks a second click that lands before the disabled button renders.
  const [isUpdatingTrigger, setIsUpdatingTrigger] = useState(false);
  const isUpdatingRef = useRef(false);

  const handleTriggerUpdate = useCallback(async () => {
    if (!selectedTrigger) return;
    if (isUpdatingRef.current) return;
    isUpdatingRef.current = true;
    setIsUpdatingTrigger(true);

    try {
      // Only save variables if they differ from deployment defaults
      const variablesToSave = getVariablesToSave();

      const { authToken } = await useUpdateTrigger(
        selectedTrigger.id,
        selectedTrigger.enabled,
        updatedEventSource === "TIME_DRIVEN" ? updatedTimeInterval : undefined,
        updatedEventSource,
        updatedDescription,
        variablesToSave,
      );

      if (authToken) {
        setIssuedAuthToken(authToken);
      } else {
        onDialogClose();
      }
    } finally {
      isUpdatingRef.current = false;
      setIsUpdatingTrigger(false);
    }
  }, [
    selectedTrigger,
    updatedEventSource,
    updatedTimeInterval,
    onDialogClose,
    useUpdateTrigger,
    updatedDescription,
    getVariablesToSave,
  ]);

  const normalizeVariables = (vars: Record<string, any> | undefined) =>
    vars && Object.keys(vars).length > 0 ? vars : {};

  const variablesChanged = !isEqual(
    normalizeVariables(getVariablesToSave()),
    normalizeVariables(selectedTrigger.variables),
  );

  const hasVariables =
    workflowVariablesObject && Object.keys(workflowVariablesObject).length > 0;
  const variableCount = workflowVariablesObject
    ? Object.keys(workflowVariablesObject).length
    : 0;

  const { hourlyDisabled, hourlyNotInPlan } = useHourlySchedule(
    selectedTrigger.workspaceId,
    selectedTrigger,
  );

  return {
    updatedEventSource,
    updatedTimeInterval,
    updatedDescription,
    variablesChanged,
    issuedAuthToken,
    handleEventSourceChange,
    handleTimeIntervalChange,
    handleTriggerUpdate,
    isUpdatingTrigger,
    handleDescriptionChange,
    workflowVariablesObject,
    pendingWorkflowData,
    openTriggerWorkflowVariablesDialog,
    setOpenTriggerWorkflowVariablesDialog,
    handleVariablesConfirm,
    hasVariables,
    variableCount,
    deploymentDefaultVariables,
    definitions,
    missingRequired,
    isLoadingVariables,
    hourlyDisabled,
    hourlyNotInPlan,
  };
};
