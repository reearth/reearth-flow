import { GraphQLError } from "graphql";
import { ClientError } from "graphql-request";
import { describe, expect, test } from "vitest";

import {
  toDuplicateVariableNames,
  toInvalidVariableValues,
  toMissingRequiredVariables,
  toUnknownVariables,
} from "./errors";

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

describe("refused deployment run variables", () => {
  test("names the variables the deployment does not define", () => {
    expect(
      toUnknownVariables(
        serverError(
          "workflow variables not defined by the deployment: aera, zone",
        ),
      ),
    ).toEqual(["aera", "zone"]);
  });

  test("names the variables given a value of the wrong type", () => {
    expect(
      toInvalidVariableValues(
        serverError("workflow variable value does not fit its type: count"),
      ),
    ).toEqual(["count"]);
  });

  // The server joins both refusals into one message, one per line.
  test("reads each list from a joined message without running into the next", () => {
    const err = serverError(
      "workflow variables not defined by the deployment: zone\nworkflow variable value does not fit its type: count, ids",
    );
    expect(toUnknownVariables(err)).toEqual(["zone"]);
    expect(toInvalidVariableValues(err)).toEqual(["count", "ids"]);
    expect(toMissingRequiredVariables(err)).toBeUndefined();
  });

  test("ignores other errors", () => {
    expect(toUnknownVariables(serverError("something else"))).toBeUndefined();
    expect(toInvalidVariableValues(new Error("boom"))).toBeUndefined();
  });
});

describe("toDuplicateVariableNames", () => {
  test("names every variable the server lists", () => {
    expect(
      toDuplicateVariableNames(
        serverError(
          "workflow variable names are used more than once: area, city",
        ),
      ),
    ).toEqual(["area", "city"]);
  });

  test("is undefined for any other error", () => {
    expect(
      toDuplicateVariableNames(serverError("something else")),
    ).toBeUndefined();
    expect(toDuplicateVariableNames(new Error("boom"))).toBeUndefined();
  });
});
