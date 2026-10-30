import { GraphQLError } from "graphql";
import { ClientError } from "graphql-request";
import { describe, expect, test } from "vitest";

import { toMissingRequiredVariables } from "./errors";

const serverError = (...messages: string[]) =>
  new ClientError(
    {
      errors: messages.map((m) => new GraphQLError(m)),
      status: 200,
      headers: new Headers(),
      body: "",
    },
    { query: "mutation CreateTrigger" },
  );

describe("toMissingRequiredVariables", () => {
  test("names every variable the server lists", () => {
    expect(
      toMissingRequiredVariables(
        serverError(
          "required workflow variables have no value: cityName, apiKey",
        ),
      ),
    ).toEqual(["cityName", "apiKey"]);
  });

  test("ignores other errors", () => {
    expect(
      toMissingRequiredVariables(serverError("something else")),
    ).toBeUndefined();
    expect(toMissingRequiredVariables(new Error("boom"))).toBeUndefined();
  });
});
