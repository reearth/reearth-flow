import { isEqual } from "lodash-es";
import { ChangeEvent, useCallback, useEffect, useState } from "react";

import { useTrigger } from "@flow/lib/gql";
import { Trigger, TimeInterval, EventSourceType } from "@flow/types";

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
  const [updatedAuthToken, setUpdatedAuthToken] = useState(
    selectedTrigger.authToken || "",
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

  const handleAuthTokenChange = useCallback(
    (e: ChangeEvent<HTMLInputElement>) => {
      setUpdatedAuthToken(e.target.value);
    },
    [],
  );

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
    openTriggerProjectVariablesDialog,
    setOpenTriggerProjectVariablesDialog,
    handleVariablesConfirm,
    getVariablesToSave,
    handleWorkflowFetch,
  } = useTriggerWorkflowVariables(selectedTrigger.variables);

  // Fetch deployment workflow to get default variables for comparison
  useEffect(() => {
    if (selectedTrigger.deployment.workflowUrl) {
      // Pass true if trigger has custom variables (don't overwrite them)
      const hasCustomVariables = !!selectedTrigger.variables;
      handleWorkflowFetch(
        selectedTrigger.deployment.workflowUrl,
        hasCustomVariables,
      );
    }
  }, [
    selectedTrigger.deployment.workflowUrl,
    selectedTrigger.variables,
    handleWorkflowFetch,
  ]);

  const handleTriggerUpdate = useCallback(async () => {
    if (!selectedTrigger) return;

    // Only save variables if they differ from deployment defaults
    const variablesToSave = getVariablesToSave();

    await useUpdateTrigger(
      selectedTrigger.id,
      updatedEventSource === "TIME_DRIVEN" ? updatedTimeInterval : undefined,
      updatedEventSource === "API_DRIVEN" ? updatedAuthToken : undefined,
      updatedDescription,
      variablesToSave,
    );

    onDialogClose();
  }, [
    selectedTrigger,
    updatedEventSource,
    updatedAuthToken,
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

  return {
    updatedEventSource,
    updatedAuthToken,
    updatedTimeInterval,
    updatedDescription,
    variablesChanged,
    handleEventSourceChange,
    handleAuthTokenChange,
    handleTimeIntervalChange,
    handleTriggerUpdate,
    handleDescriptionChange,
    workflowVariablesObject,
    pendingWorkflowData,
    openTriggerProjectVariablesDialog,
    setOpenTriggerProjectVariablesDialog,
    handleVariablesConfirm,
    hasVariables,
    variableCount,
  };
};
