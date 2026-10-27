import { GraphQLError } from "graphql";
import { ClientError } from "graphql-request";
import { describe, expect, test } from "vitest";

import { toPlanLimitKind } from "./errors";

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
  ])("%s", (message, kind) => {
    expect(toPlanLimitKind(serverError(message))).toBe(kind);
  });

  test("leaves any other error unclassified", () => {
    expect(toPlanLimitKind(serverError("failed to submit job"))).toBe(
      undefined,
    );
    expect(toPlanLimitKind(new Error("network down"))).toBe(undefined);
  });
});
