import { describe, expect, it } from "vitest";

import { toGqlVariableInput } from "./useQueries";

describe("toGqlVariableInput", () => {
  it("sends a known type as its server type", () => {
    expect(
      toGqlVariableInput([{ key: "area", type: "choice", value: "tokyo" }]),
    ).toEqual([{ key: "area", type: "CHOICE", value: "tokyo" }]);
  });

  it("keeps a variable whose type the server has no counterpart for, as text", () => {
    // A URL value is inferred as web_connection, which the server lacks.
    expect(
      toGqlVariableInput([
        {
          key: "source",
          type: "web_connection",
          value: "https://example.com/data.zip",
        },
        { key: "apiKey", type: "password", value: "secret" },
      ]),
    ).toEqual([
      { key: "source", type: "TEXT", value: "https://example.com/data.zip" },
      { key: "apiKey", type: "TEXT", value: "secret" },
    ]);
  });
});
