import { ChangeEvent, useCallback, useState } from "react";

import { useTrigger } from "@flow/lib/gql";
import { Trigger, TimeInterval, EventSourceType } from "@flow/types";

export default ({
  selectedTrigger,
  onDialogClose,
}: {
  selectedTrigger: Trigger;
  onDialogClose: () => void;
}) => {
  const { useUpdateTrigger } = useTrigger();

  const [updatedEventSource, setUpdatedEventSource] = useState(
    selectedTrigger.eventSource,
  );
  const [updatedAuthToken, setUpdatedAuthToken] = useState(
    selectedTrigger.authToken || "",
  );
  const [updatedTimeInterval, setUpdatedTimeInterval] = useState<
    TimeInterval | undefined
  >(selectedTrigger.timeInterval || undefined);

  const handleEventSourceChange = useCallback(
    (eventSource: EventSourceType) => {
      setUpdatedEventSource(eventSource);
      if (eventSource === "API_DRIVEN") {
        setUpdatedTimeInterval(undefined);
      }
      if (eventSource === "TIME_DRIVEN") {
        setUpdatedAuthToken("");
      }
    },
    [],
  );

  const handleAuthTokenChange = useCallback(
    (e: ChangeEvent<HTMLInputElement>) => {
      setUpdatedAuthToken(e.target.value);
    },
    [],
  );

  const handleTimeIntervalChange = useCallback((timeInterval: TimeInterval) => {
    setUpdatedTimeInterval(timeInterval);
  }, []);

  const handleTriggerUpdate = useCallback(async () => {
    if (!selectedTrigger) return;

    await useUpdateTrigger(
      selectedTrigger.id,
      updatedEventSource === "TIME_DRIVEN" ? updatedTimeInterval : undefined,
      updatedEventSource === "API_DRIVEN" ? updatedAuthToken : undefined,
    );

    onDialogClose();
  }, [
    selectedTrigger,
    updatedEventSource,
    updatedAuthToken,
    updatedTimeInterval,
    onDialogClose,
    useUpdateTrigger,
  ]);

  return {
    updatedEventSource,
    updatedAuthToken,
    updatedTimeInterval,
    handleEventSourceChange,
    handleAuthTokenChange,
    handleTimeIntervalChange,
    handleTriggerUpdate,
  };
};
