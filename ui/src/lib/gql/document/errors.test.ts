import { GraphQLError } from "graphql";
import { ClientError } from "graphql-request";
import { describe, expect, test } from "vitest";

import { isVersionNotRebuildable } from "./errors";

const serverError = (message: string, code?: string) =>
  new ClientError(
    {
      errors: [new GraphQLError(message, code ? { extensions: { code } } : {})],
      status: 200,
      headers: new Headers(),
      body: "",
    },
    { query: "mutation RollbackProject" },
  );

describe("isVersionNotRebuildable", () => {
  test("recognizes the code, whatever the message says", () => {
    expect(
      isVersionNotRebuildable(
        serverError("reworded later", "VERSION_NOT_REBUILDABLE"),
      ),
    ).toBe(true);
  });

  test("does not mistake another refusal for it", () => {
    expect(
      isVersionNotRebuildable(serverError("denied", "OPERATION_DENIED")),
    ).toBe(false);
  });

  test("does not mistake an outage for it", () => {
    expect(isVersionNotRebuildable(new Error("network down"))).toBe(false);
  });
});
