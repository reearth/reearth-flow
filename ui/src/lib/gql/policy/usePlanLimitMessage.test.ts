import { renderHook } from "@testing-library/react";
import { GraphQLError } from "graphql";
import { ClientError } from "graphql-request";
import { describe, expect, test } from "vitest";

import { usePlanLimitMessage } from "./usePlanLimitMessage";

const serverError = (message: string) =>
  new ClientError(
    {
      errors: [new GraphQLError(message)],
      status: 200,
      headers: new Headers(),
      body: "",
    },
    { query: "mutation RunProject" },
  );

describe("usePlanLimitMessage", () => {
  test("explains a run refused for the job limit", () => {
    const { result } = renderHook(() => usePlanLimitMessage());

    expect(
      result.current(
        serverError(
          "this workspace already has as many jobs running as its plan allows: 01jabc",
        ),
      ),
    ).toContain("already running as many debug runs as its plan allows");
  });

  test.each([
    [
      "this workspace's plan does not allow more debug runs this hour",
      "all the debug runs its plan allows this hour",
    ],
    [
      "this workspace's plan does not allow more deployment runs today",
      "all the deployment runs its plan allows today",
    ],
    [
      "this workspace already has as many triggers enabled as its plan allows",
      "Disable another trigger first",
    ],
  ])("explains %j", (message, expected) => {
    const { result } = renderHook(() => usePlanLimitMessage());

    expect(result.current(serverError(message))).toContain(expected);
  });

  test("names the plan's asset size limit when the server sends it", () => {
    const { result } = renderHook(() => usePlanLimitMessage());

    expect(
      result.current(
        serverError(
          "this asset is larger than the workspace's plan allows: the limit is 5MB",
        ),
      ),
    ).toContain("The limit is 5MB");
    expect(
      result.current(
        serverError("this asset is larger than the workspace's plan allows"),
      ),
    ).toBe("This file is larger than this workspace's plan allows.");
  });

  test("explains a refused hourly schedule", () => {
    const { result } = renderHook(() => usePlanLimitMessage());

    expect(
      result.current(
        serverError("this workspace's plan does not allow hourly schedules"),
      ),
    ).toContain("doesn't include hourly schedules");
  });

  test("leaves other errors to the caller", () => {
    const { result } = renderHook(() => usePlanLimitMessage());

    expect(result.current(serverError("failed to submit job"))).toBe(undefined);
  });
});
