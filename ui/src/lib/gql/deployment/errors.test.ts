import { GraphQLError } from "graphql";
import { ClientError } from "graphql-request";
import { describe, expect, test } from "vitest";

import { isDeploymentHasTriggersError } from "./errors";

const serverError = (message: string, code?: string) =>
  new ClientError(
    {
      errors: [new GraphQLError(message, code ? { extensions: { code } } : {})],
      status: 200,
      headers: new Headers(),
      body: "",
    },
    { query: "mutation DeleteDeployment" },
  );

describe("isDeploymentHasTriggersError", () => {
  test("recognizes the code", () => {
    expect(
      isDeploymentHasTriggersError(
        serverError("reworded later", "DEPLOYMENT_HAS_TRIGGERS"),
      ),
    ).toBe(true);
  });

  test("recognizes the message from an API without the code", () => {
    expect(
      isDeploymentHasTriggersError(
        serverError("deployment has active triggers and cannot be deleted"),
      ),
    ).toBe(true);
  });

  test("is false for any other error", () => {
    expect(
      isDeploymentHasTriggersError(
        serverError("operation denied", "OPERATION_DENIED"),
      ),
    ).toBe(false);
    expect(isDeploymentHasTriggersError(new Error("network"))).toBe(false);
  });
});
