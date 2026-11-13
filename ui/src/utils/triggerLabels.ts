import type { EventSourceType, TimeInterval } from "@flow/types";

// The event sources the trigger dialogs offer. CMS triggers are run from CMS
// and are not offered until the dialogs support them.
export type SelectableEventSource = Exclude<EventSourceType, "CMS">;

export const getEventSourceLabels = (
  t: (key: string) => string,
): Record<SelectableEventSource, string> => ({
  API_DRIVEN: t("API Driven"),
  TIME_DRIVEN: t("Time Driven"),
});

export const getTimeIntervalLabels = (
  t: (key: string) => string,
): Record<TimeInterval, string> => ({
  EVERY_DAY: t("Every Day"),
  EVERY_HOUR: t("Every Hour"),
  EVERY_WEEK: t("Every Week"),
  EVERY_MONTH: t("Every Month"),
});

export const getEventSourceLabel = (
  eventSource: EventSourceType | undefined,
  t: (key: string) => string,
): string | undefined =>
  eventSource &&
  (eventSource === "CMS"
    ? t("CMS")
    : (getEventSourceLabels(t)[eventSource] ?? eventSource));

export const getTimeIntervalLabel = (
  timeInterval: TimeInterval | undefined,
  t: (key: string) => string,
): string | undefined =>
  timeInterval && (getTimeIntervalLabels(t)[timeInterval] ?? timeInterval);
