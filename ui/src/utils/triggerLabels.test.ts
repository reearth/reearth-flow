import { describe, expect, test } from "vitest";

import {
  getEventSourceLabel,
  getEventSourceLabels,
  isEventSource,
} from "./triggerLabels";

const t = (key: string) => key;

describe("getEventSourceLabels", () => {
  test("offers every event source, CMS last", () => {
    expect(Object.keys(getEventSourceLabels(t))).toEqual([
      "API_DRIVEN",
      "TIME_DRIVEN",
      "CMS",
    ]);
  });
});

describe("getEventSourceLabel", () => {
  test("labels a CMS trigger", () => {
    expect(getEventSourceLabel("CMS", t)).toBe("CMS");
  });

  test("is unset without an event source", () => {
    expect(getEventSourceLabel(undefined, t)).toBeUndefined();
  });
});

describe("isEventSource", () => {
  test("accepts the event sources and nothing else", () => {
    expect(isEventSource("API_DRIVEN")).toBe(true);
    expect(isEventSource("TIME_DRIVEN")).toBe(true);
    expect(isEventSource("CMS")).toBe(true);
    expect(isEventSource("")).toBe(false);
    expect(isEventSource("cms")).toBe(false);
  });
});
