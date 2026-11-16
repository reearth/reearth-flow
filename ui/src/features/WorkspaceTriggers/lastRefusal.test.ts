import { describe, expect, test } from "vitest";

import type { TriggerRefusal } from "@flow/types";

import { isRetrying } from "./lastRefusal";

const now = new Date("2026-11-06T01:00:00Z");
const refusal = (retryUntil?: string): TriggerRefusal => ({
  refusedAt: "2026-11-06T00:03:00Z",
  code: "CONCURRENT_JOB_LIMIT",
  retryUntil,
});

describe("isRetrying", () => {
  test("before its retries end", () => {
    expect(isRetrying(refusal("2026-11-06T02:33:00Z"), now)).toBe(true);
  });

  test("once its retries have ended", () => {
    expect(isRetrying(refusal("2026-11-06T00:49:00Z"), now)).toBe(false);
  });

  test("a run that was skipped", () => {
    expect(isRetrying(refusal(), now)).toBe(false);
  });
});
