import { ChangeEvent, useCallback, useState } from "react";

import { useTrigger } from "@flow/lib/gql";
import { Trigger, TimeInterval, EventSourceType } from "@flow/types";

import { useDeploymentWorkflowVariables } from "../TriggerWorkflowVariables/useDeploymentWorkflowVariables";

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
    initializeVariables,
  } = useDeploymentWorkflowVariables(selectedTrigger.variables);

  const handleTriggerUpdate = useCallback(async () => {
    if (!selectedTrigger) return;

    await useUpdateTrigger(
      selectedTrigger.id,
      updatedEventSource === "TIME_DRIVEN" ? updatedTimeInterval : undefined,
      updatedEventSource === "API_DRIVEN" ? updatedAuthToken : undefined,
      updatedDescription,
      workflowVariablesObject,
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
    workflowVariablesObject,
  ]);

  // Check if variables have changed by comparing JSON strings
  const variablesChanged =
    JSON.stringify(workflowVariablesObject || {}) !==
    JSON.stringify(selectedTrigger.variables || {});

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
    initializeVariables,
  };
};
