import type { EventSourceType, TimeInterval } from "@flow/types";

export const getEventSourceLabels = (
  t: (key: string) => string,
): Record<EventSourceType, string> => ({
  API_DRIVEN: t("API Driven"),
  TIME_DRIVEN: t("Time Driven"),
  CMS: t("CMS"),
});

const eventSources: readonly EventSourceType[] = [
  "API_DRIVEN",
  "TIME_DRIVEN",
  "CMS",
];

export const isEventSource = (value: string): value is EventSourceType =>
  (eventSources as readonly string[]).includes(value);

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
  eventSource && (getEventSourceLabels(t)[eventSource] ?? eventSource);

export const getTimeIntervalLabel = (
  timeInterval: TimeInterval | undefined,
  t: (key: string) => string,
): string | undefined =>
  timeInterval && (getTimeIntervalLabels(t)[timeInterval] ?? timeInterval);
