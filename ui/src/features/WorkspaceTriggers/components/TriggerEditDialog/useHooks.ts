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
