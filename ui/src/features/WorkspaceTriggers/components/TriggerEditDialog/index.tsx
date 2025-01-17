import { useState } from "react";

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
import { Trigger, EventSourceType, TimeInterval } from "@flow/types/trigger";

type Props = {
  selectedTrigger: Trigger;
  onDialogClose: () => void;
};

const TriggerEditDialog: React.FC<Props> = ({
  selectedTrigger,
  onDialogClose,
}) => {
  const t = useT();

  const [eventSource, setEventSource] = useState<EventSourceType>(
    selectedTrigger.eventSource,
  );

  const [authToken, setAuthToken] = useState<string>(
    selectedTrigger.authToken || "",
  );
  const [timeInterval, setTimeInterval] = useState<TimeInterval | null>(
    selectedTrigger.timeInterval || null,
  );

  const eventSources: Record<EventSourceType, string> = {
    API_DRIVEN: t("API Driven"),
    TIME_DRIVEN: t("Time Driven"),
  };

  const timeIntervals: Record<TimeInterval, string> = {
    EVERY_DAY: t("Every Day"),
    EVERY_HOUR: t("Every Hour"),
    EVERY_WEEK: t("Every Week"),
    EVERY_MONTH: t("Every Month"),
  };

  const handleUpdateTrigger = () => {
    console.log("Updated Event Source:", eventSource);
    console.log("Updated Time Interval:", timeInterval);
  };

  return (
    <Dialog open={true} onOpenChange={onDialogClose}>
      <DialogContent size="sm">
        <DialogTitle>{t("Edit Trigger")}</DialogTitle>
        <DialogContentWrapper>
          <div className="border-b border-primary text-center" />
          <DialogContentSection className="flex-1">
            <Label htmlFor="event-source-selector">
              {t("Select Event Source")}
            </Label>
            <Select
              value={eventSource}
              onValueChange={(value) => {
                setEventSource(value as EventSourceType);
                // Reset time interval if switching to API_DRIVEN
                if (value === EventSourceType.API_DRIVEN) {
                  setTimeInterval(null);
                }
              }}>
              <SelectTrigger>
                <SelectValue placeholder={eventSources[eventSource]} />
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
          {eventSource === EventSourceType.TIME_DRIVEN && (
            <DialogContentSection className="flex-1">
              <Label htmlFor="time-interval-selector">
                {t("Select Time Interval")}
              </Label>
              <Select
                value={timeInterval || ""}
                onValueChange={(value) =>
                  setTimeInterval(value as TimeInterval)
                }>
                <SelectTrigger>
                  <SelectValue
                    placeholder={
                      timeInterval
                        ? timeIntervals[timeInterval]
                        : timeIntervals.EVERY_DAY
                    }
                  />
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
          <Button onClick={handleUpdateTrigger}>{t("Update Trigger")}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export { TriggerEditDialog };
