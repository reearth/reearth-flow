import { GraphQLError } from "graphql";
import { ClientError } from "graphql-request";
import { describe, expect, it } from "vitest";

import { snapshotErrorReason } from "./hooks";

describe("snapshotErrorReason", () => {
  it("treats a GraphQL response error as unavailable", () => {
    const error = new ClientError(
      {
        errors: [new GraphQLError("operation denied")],
        status: 200,
        headers: new Headers(),
        body: "",
      },
      { query: "query GetLatestProjectSnapshot" },
    );
    expect(snapshotErrorReason(error)).toBe("unavailable");
  });

  it("treats a failed request as a network failure", () => {
    expect(snapshotErrorReason(new TypeError("Failed to fetch"))).toBe(
      "network",
    );
  });
});
