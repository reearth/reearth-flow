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
    ).toContain("already running as many jobs as its plan allows");
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
