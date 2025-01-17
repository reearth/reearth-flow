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
import { useCurrentWorkspace } from "@flow/stores";
import { EventSourceType, TimeInterval } from "@flow/types/trigger";

type Props = {
  setShowDialog: (show: boolean) => void;
};

const TriggerAddDialog: React.FC<Props> = ({ setShowDialog }) => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  const [eventSource, setEventSource] = useState<EventSourceType>(
    EventSourceType.API_DRIVEN,
  );
  const [timeInterval, setTimeInterval] = useState<TimeInterval | null>(null);

  const [authToken, setAuthToken] = useState<string>("");
  const handleSelectEventSource = (eventSource: EventSourceType) => {
    if (eventSource === "API_DRIVEN") {
      setTimeInterval(null);
    }
    setEventSource(eventSource);
  };

  const eventSources: Record<EventSourceType, string> = {
    API_DRIVEN: t("API Driven"),
    TIME_DRIVEN: t("Time Driven"),
  };

  const handleSelectTimeInterval = (timeInterval: TimeInterval) => {
    setTimeInterval(timeInterval);
  };

  const timeIntervals: Record<TimeInterval, string> = {
    EVERY_DAY: t("Every Day"),
    EVERY_HOUR: t("Every Hour"),
    EVERY_WEEK: t("Every Week"),
    EVERY_MONTH: t("Every Month"),
  };

  console.log("tEST", eventSource, timeInterval);
  return (
    <Dialog open={true} onOpenChange={() => setShowDialog(false)}>
      <DialogContent size="sm">
        <DialogTitle>{t("Create a new trigger")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection className="flex-1">
            <Label htmlFor="event-source-selector">
              {t("Select Event Source")}
            </Label>
            <Select onValueChange={handleSelectEventSource}>
              <SelectTrigger>
                <SelectValue placeholder={eventSources.API_DRIVEN} />
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
              <Select onValueChange={handleSelectTimeInterval}>
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
          <Button>{t("Add New Trigger")}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export { TriggerAddDialog };
