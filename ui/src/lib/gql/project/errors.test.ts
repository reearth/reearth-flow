import { GraphQLError } from "graphql";
import { ClientError } from "graphql-request";
import { describe, expect, test } from "vitest";

import { toProjectErrorKind } from "./errors";

const serverError = (...messages: string[]) =>
  new ClientError(
    {
      errors: messages.map((m) => new GraphQLError(m)),
      status: 200,
      headers: new Headers(),
      body: "",
    },
    { query: "mutation UpdateProject" },
  );

describe("toProjectErrorKind", () => {
  test.each([
    [
      "this workspace's plan does not allow private projects",
      "privateNotAllowed",
    ],
    ["project alias is already used by another project", "aliasTaken"],
    ["invalid alias: the 'p-' prefix is reserved", "aliasReserved"],
    ["invalid alias: use 5-32 letters, digits, '-' or '_'", "aliasInvalid"],
    ["too many topics", "tooManyTopics"],
    ["topic is too long", "topicTooLong"],
  ])("classifies %j", (message, kind) => {
    expect(toProjectErrorKind(serverError(message))).toBe(kind);
  });

  test("classifies a message the server wrapped", () => {
    expect(
      toProjectErrorKind(
        serverError(
          "transaction failed: this workspace's plan does not allow private projects",
        ),
      ),
    ).toBe("privateNotAllowed");
  });

  test("leaves anything else unclassified", () => {
    expect(toProjectErrorKind(serverError("operation denied"))).toBeUndefined();
    expect(toProjectErrorKind(new Error("network down"))).toBeUndefined();
  });
});
