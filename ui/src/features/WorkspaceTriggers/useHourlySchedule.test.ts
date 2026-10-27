import { renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";

import type { Trigger } from "@flow/types";

import { useHourlySchedule } from "./useHourlySchedule";

let canScheduleHourly: boolean | undefined;

vi.mock("@flow/lib/gql", () => ({
  useWorkspace: () => ({
    useGetWorkspacePolicy: (workspaceId: string) => ({
      policy:
        canScheduleHourly === undefined
          ? undefined
          : { workspaceId, canCreatePrivateProject: true, canScheduleHourly },
    }),
  }),
}));

const trigger = (overrides: Partial<Trigger>) =>
  ({
    id: "t1",
    workspaceId: "ws",
    eventSource: "TIME_DRIVEN",
    timeInterval: "EVERY_DAY",
    ...overrides,
  }) as Trigger;

describe("useHourlySchedule", () => {
  beforeEach(() => {
    canScheduleHourly = undefined;
  });

  test("a plan without hourly schedules greys the option out and says why", () => {
    canScheduleHourly = false;
    const { result } = renderHook(() => useHourlySchedule("ws"));

    expect(result.current).toEqual({
      hourlyDisabled: true,
      hourlyNotInPlan: true,
    });
  });

  test("a plan with hourly schedules offers the option", () => {
    canScheduleHourly = true;
    const { result } = renderHook(() => useHourlySchedule("ws"));

    expect(result.current).toEqual({
      hourlyDisabled: false,
      hourlyNotInPlan: false,
    });
  });

  test("the option stays unavailable, without a message, until the policy loads", () => {
    const { result } = renderHook(() => useHourlySchedule("ws"));

    expect(result.current).toEqual({
      hourlyDisabled: true,
      hourlyNotInPlan: false,
    });
  });

  test("a trigger already hourly keeps the option", () => {
    canScheduleHourly = false;
    const { result } = renderHook(() =>
      useHourlySchedule("ws", trigger({ timeInterval: "EVERY_HOUR" })),
    );

    expect(result.current).toEqual({
      hourlyDisabled: false,
      hourlyNotInPlan: false,
    });
  });

  test("an API trigger switched to a schedule gets no exception", () => {
    canScheduleHourly = false;
    const { result } = renderHook(() =>
      useHourlySchedule(
        "ws",
        trigger({ eventSource: "API_DRIVEN", timeInterval: undefined }),
      ),
    );

    expect(result.current.hourlyDisabled).toBe(true);
  });
});
