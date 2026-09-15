import { readFileSync } from "fs";

import { describe, expect, it } from "vitest";

import { compile } from "./compile";
import { applyDefaults } from "./defaults";
import { isValid, validate } from "./validate";

const actions = JSON.parse(
  readFileSync("../engine/schema/actions.json", "utf8"),
).actions as { name: string; parameter?: unknown }[];

const schemaFor = (name: string) => {
  const action = actions.find((entry) => entry.name === name);
  if (!action?.parameter) throw new Error(`${name} has no parameter schema`);
  return action.parameter as never;
};

const seed = (name: string, value: unknown) =>
  applyDefaults(compile(schemaFor(name), { actionName: name }), value);

describe("applyDefaults", () => {
  it("fills only what HTTP Caller's schema states", () => {
    // The old pipeline produced url/authentication/requestBody/response/retry/
    // rateLimit here, none of which the schema asks for.
    expect(seed("HTTP Caller", {})).toEqual({ method: "GET" });
  });

  it("leaves every optional section absent", () => {
    const seeded = seed("HTTP Caller", {}) as Record<string, unknown>;
    for (const name of [
      "authentication",
      "requestBody",
      "response",
      "retry",
      "rateLimit",
      "timeouts",
      "httpOptions",
    ]) {
      expect(seeded).not.toHaveProperty(name);
    }
  });

  it("invents no responseEncoding", () => {
    const seeded = seed("HTTP Caller", { response: {} }) as {
      response: Record<string, unknown>;
    };
    expect(seeded.response).not.toHaveProperty("responseEncoding");
    // The one default the schema does state is applied.
    expect(seeded.response.responseBodyAttribute).toBe("_response_body");
  });

  it("fills a section's defaults once the user opts into it", () => {
    const seeded = seed("HTTP Caller", { rateLimit: { requests: 5 } }) as {
      rateLimit: Record<string, unknown>;
    };
    expect(seeded.rateLimit).toEqual({
      requests: 5,
      intervalMs: 1000,
      timing: "burst",
    });
  });

  it("preserves an explicit null rather than replacing it", () => {
    const seeded = seed("HTTP Caller", {
      response: { responseEncoding: null },
    }) as { response: Record<string, unknown> };
    expect(seeded.response.responseEncoding).toBeNull();
  });

  it("keeps a union's discriminator", () => {
    expect(
      seed("Feature Writer", {
        format: { type: "json" },
        output: { type: "string", value: "o.json" },
      }),
    ).toEqual({
      format: { type: "json", converter: null },
      output: { type: "string", value: "o.json" },
    });
  });

  it("seeds defaults inside array items", () => {
    const node = compile({
      type: "object",
      properties: {
        rules: {
          type: "array",
          items: {
            type: "object",
            properties: { mode: { type: "string", default: "keep" } },
          },
        },
      },
    } as never);
    expect(applyDefaults(node, { rules: [{}, { mode: "drop" }] })).toEqual({
      rules: [{ mode: "keep" }, { mode: "drop" }],
    });
  });

  it("adds no validation error, for every action", () => {
    // The property that matters: seeding may fill a field in, never break one.
    // Errors already present in the empty input — a required field the user has
    // not typed yet, a root union with no variant chosen — are expected and are
    // the user's to resolve.
    const failures: string[] = [];

    for (const action of actions) {
      if (!action.parameter) continue;
      const schema = action.parameter as never;
      const before = validate(schema, {});
      const seeded = applyDefaults(
        compile(schema, { actionName: action.name }),
        {},
      );
      const after = validate(schema, seeded ?? {});

      const introduced = Object.entries(after).filter(
        ([key, messages]) =>
          before[key] === undefined ||
          messages.some((message) => !before[key].includes(message)),
      );
      if (introduced.length > 0) {
        failures.push(`${action.name}: ${JSON.stringify(introduced)}`);
      }
    }

    expect(failures).toEqual([]);
  });
});

describe("a parameterless action", () => {
  // `parameter: null` for 30 of the 171 actions, handed straight to `compile`
  // by `buildNewCanvasNode` every time one is dropped on the canvas.
  it("compiles and seeds without throwing", () => {
    for (const schema of [null, undefined]) {
      expect(() => applyDefaults(compile(schema), undefined)).not.toThrow();
      expect(applyDefaults(compile(schema), undefined)).toBeUndefined();
    }
  });

  it("covers every action the engine publishes without parameters", () => {
    const parameterless = actions.filter((action) => !action.parameter);
    expect(parameterless.length).toBeGreaterThan(0);
    for (const action of parameterless) {
      expect(() =>
        applyDefaults(
          compile(action.parameter as never, { actionName: action.name }),
          undefined,
        ),
      ).not.toThrow();
    }
  });
});

describe("validate", () => {
  it("agrees with the engine about a nullable enum", () => {
    const schema = schemaFor("HTTP Caller");
    expect(
      isValid(
        validate(schema, {
          url: { type: "string", value: "https://x.test" },
          response: { responseEncoding: null },
        }),
      ),
    ).toBe(true);
  });

  it("puts a missing required property on the property, not its parent", () => {
    const errors = validate(schemaFor("HTTP Caller"), {});
    // Present, so the field is marked invalid; empty, because the asterisk and
    // the red border already say it and a sentence would be the third telling.
    expect(errors).toHaveProperty("url");
    expect(errors.url).toEqual([]);
  });

  it("reports a genuinely wrong enum value", () => {
    const errors = validate(schemaFor("HTTP Caller"), {
      url: { type: "string", value: "https://x.test" },
      response: { responseEncoding: "utf-16" },
    });
    expect(Object.keys(errors)).toContain("response.responseEncoding");
  });

  it("locates an error inside an array item", () => {
    const errors = validate(
      {
        type: "object",
        properties: {
          rules: {
            type: "array",
            items: {
              type: "object",
              required: ["name"],
              properties: { name: { type: "string" } },
            },
          },
        },
      } as never,
      { rules: [{ name: "ok" }, {}] },
    );
    expect(errors).toHaveProperty("rules.1.name");
    expect(errors["rules.1.name"]).toEqual([]);
  });
});
