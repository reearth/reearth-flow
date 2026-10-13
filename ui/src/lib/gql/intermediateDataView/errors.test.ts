import { GraphQLError } from "graphql";
import { ClientError } from "graphql-request";
import { describe, expect, test } from "vitest";

import {
  IntermediateDataViewError,
  toIntermediateDataViewError,
} from "./errors";

const serverError = (...messages: string[]) =>
  new ClientError(
    {
      errors: messages.map((m) => new GraphQLError(m)),
      status: 200,
      headers: new Headers(),
      body: "",
    },
    { query: "mutation RenderIntermediateDataTilesView" },
  );

describe("toIntermediateDataViewError", () => {
  test.each([
    ["the view did not finish rendering in time", "timedOut"],
    ["intermediate data views are not available", "unavailable"],
    ["operation denied", "permissionDenied"],
  ])("classifies %j", (message, kind) => {
    expect(toIntermediateDataViewError(serverError(message)).kind).toBe(kind);
  });

  test("classifies an error the server wrapped with detail", () => {
    // The server wraps these with the job or port it was about, so only the
    // start of the message is fixed.
    expect(
      toIntermediateDataViewError(
        serverError("job has not finished: 0a1b is running"),
      ).kind,
    ).toBe("jobNotFinished");
    expect(
      toIntermediateDataViewError(
        serverError("intermediate data not found: n1.default on job 0a1b"),
      ).kind,
    ).toBe("dataNotFound");
  });

  test("keeps the server's message", () => {
    const err = toIntermediateDataViewError(
      serverError("job has not finished: 0a1b is running"),
    );
    expect(err.message).toBe("job has not finished: 0a1b is running");
  });

  test("answers an unrecognised server error as unknown, with its message", () => {
    const err = toIntermediateDataViewError(serverError("internal error"));
    expect(err.kind).toBe("unknown");
    expect(err.message).toBe("internal error");
  });

  test("finds a recognised message among several", () => {
    expect(
      toIntermediateDataViewError(
        serverError("something else", "operation denied"),
      ).kind,
    ).toBe("permissionDenied");
  });

  test("answers a network failure as unknown", () => {
    const err = toIntermediateDataViewError(new TypeError("Failed to fetch"));
    expect(err.kind).toBe("unknown");
    expect(err.message).toBe("Failed to fetch");
  });

  test("passes an already classified error through", () => {
    const original = new IntermediateDataViewError("tooManyRenders");
    expect(toIntermediateDataViewError(original)).toBe(original);
  });
});
