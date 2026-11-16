import { GraphQLError } from "graphql";
import { ClientError } from "graphql-request";
import { describe, expect, test } from "vitest";

import { toPlanLimit, toPlanLimitKind } from "./errors";

const serverError = (...messages: string[]) =>
  new ClientError(
    {
      errors: messages.map((m) => new GraphQLError(m)),
      status: 200,
      headers: new Headers(),
      body: "",
    },
    { query: "mutation RunProject" },
  );

describe("toPlanLimitKind", () => {
  test.each([
    [
      "this workspace already has as many jobs running as its plan allows",
      "jobLimitReached",
    ],
    [
      "this workspace already has as many jobs running as its plan allows: 01jabc, 01jdef",
      "jobLimitReached",
    ],
    [
      "this workspace's plan does not allow hourly schedules",
      "hourlyScheduleNotAllowed",
    ],
    [
      "this workspace's plan does not allow more debug runs this hour",
      "debugRunsPerHourLimit",
    ],
    [
      "this workspace's plan does not allow more deployment runs today",
      "deploymentRunsPerDayLimit",
    ],
    [
      "this workspace already has as many triggers enabled as its plan allows",
      "enabledTriggerLimit",
    ],
    [
      "this asset is larger than the workspace's plan allows: the limit is 5MB",
      "assetTooLarge",
    ],
  ])("%s", (message, kind) => {
    expect(toPlanLimitKind(serverError(message))).toBe(kind);
  });

  test("reads the plan's limit from the asset size refusal", () => {
    expect(
      toPlanLimit(
        serverError(
          "this asset is larger than the workspace's plan allows: the limit is 5MB",
        ),
      ),
    ).toEqual({ kind: "assetTooLarge", limit: "5MB" });
    expect(
      toPlanLimit(
        serverError("this asset is larger than the workspace's plan allows"),
      ),
    ).toEqual({ kind: "assetTooLarge" });
  });

  test("leaves any other error unclassified", () => {
    expect(toPlanLimitKind(serverError("failed to submit job"))).toBe(
      undefined,
    );
    expect(toPlanLimitKind(new Error("network down"))).toBe(undefined);
  });
});
