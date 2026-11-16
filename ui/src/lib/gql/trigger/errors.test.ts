import { GraphQLError } from "graphql";
import { ClientError } from "graphql-request";
import { describe, expect, test } from "vitest";

import { isScheduleNotUpdated } from "./errors";

const serverError = (...messages: string[]) =>
  new ClientError(
    {
      errors: messages.map((m) => new GraphQLError(m)),
      status: 200,
      headers: new Headers(),
      body: "",
    },
    { query: "mutation UpdateTrigger" },
  );

describe("isScheduleNotUpdated", () => {
  test("recognises an edit saved without its schedule", () => {
    expect(
      isScheduleNotUpdated(
        serverError(
          "the trigger was saved, but its schedule could not be updated; save it again to retry",
        ),
      ),
    ).toBe(true);
  });

  test("leaves any other error alone", () => {
    expect(isScheduleNotUpdated(serverError("trigger not found"))).toBe(false);
    expect(isScheduleNotUpdated(new Error("network down"))).toBe(false);
  });
});
