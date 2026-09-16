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

  it("leaves a null root null rather than seeding an object into it", () => {
    // `null` is a stated value, not an absence. Turning it into `{}` made a
    // root the schema legally allows to be null report as invalid.
    const node = compile({
      anyOf: [
        { type: "object", properties: { a: { type: "string", default: "x" } } },
        { type: "null" },
      ],
    } as never);
    expect(applyDefaults(node, null)).toBeNull();
    // Absent is still absent: a nullable section is not conjured into
    // existence just to carry its defaults. `SchemaForm` validates an absent
    // root as `{}`, which is where the required-field errors come from.
    expect(applyDefaults(node, undefined)).toBeUndefined();
    // The same schema without its null branch does seed, since it must exist.
    const required = compile({
      type: "object",
      properties: { a: { type: "string", default: "x" } },
    } as never);
    expect(applyDefaults(required, undefined)).toEqual({ a: "x" });
  });

  it("leaves a null section null instead of filling its defaults in", () => {
    expect(seed("HTTP Caller", { rateLimit: null })).toEqual({
      method: "GET",
      rateLimit: null,
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

describe("a schema that permits only null", () => {
  // `must be null` is dropped where it is the null branch of a nullable union
  // reporting on a section that is present but incomplete. It is a real
  // complaint when null is the only value the schema allows, and dropping it
  // there left the form with no errors at all and a verdict of valid.
  it("reports a wrong value against a standalone null schema", () => {
    expect(isValid(validate({ type: "null" } as never, "oops"))).toBe(false);
    expect(isValid(validate({ type: "null" } as never, null))).toBe(true);
  });

  it("reports a wrong value against a null-only property", () => {
    const schema = {
      type: "object",
      properties: { a: { type: "null" } },
    } as never;
    expect(validate(schema, { a: "oops" })).toHaveProperty("a");
    expect(isValid(validate(schema, { a: null }))).toBe(true);
  });

  it("still says nothing for the null branch of a nullable union", () => {
    const schema = {
      type: "object",
      properties: {
        a: {
          anyOf: [
            {
              type: "object",
              required: ["x"],
              properties: { x: { type: "string" } },
            },
            { type: "null" },
          ],
        },
      },
    } as never;
    const errors = validate(schema, { a: {} });
    // The child's missing field is the complaint; "must be null" is not.
    expect(Object.values(errors).flat()).not.toContain("must be null");
    expect(errors).toHaveProperty("a.x");
  });

  it("keeps the section's own error when nothing more specific is known", () => {
    const schema = {
      type: "object",
      properties: {
        a: { anyOf: [{ type: "string" }, { type: "null" }] },
      },
    } as never;
    // A number is neither a string nor null: the field itself is what is wrong.
    expect(isValid(validate(schema, { a: 5 }))).toBe(false);
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
