/**
 * Migration carries a stored value across a schema change.
 *
 * The rule is one-directional: keep what the new schema can represent, drop
 * what it cannot, invent nothing.
 */
import { readFileSync } from "fs";

import { describe, expect, it } from "vitest";

import { compile } from "./compile";
import { migrateValue } from "./migrate";
import { validate } from "./validate";

const actions = JSON.parse(
  readFileSync("../engine/schema/actions.json", "utf8"),
).actions as { name: string; parameter?: unknown }[];

const schemaFor = (name: string) => {
  const action = actions.find((entry) => entry.name === name);
  if (!action?.parameter) throw new Error(`${name} has no parameter schema`);
  return action.parameter;
};

const migrate = (schema: unknown, stored: unknown, actionName?: string) =>
  migrateValue(compile(schema as never, { actionName }), stored);

describe("a schema whose root is not an object", () => {
  // Reading the new schema's top-level `properties` found nothing for these,
  // so every stored value was discarded before the user saw the form.
  it("keeps the stored value for a union at the root", () => {
    // JSON Fragmenter's root is a union tagged by `inputSource`.
    const stored = { inputSource: "attribute", jsonAttribute: "payload" };
    const migrated = migrate(schemaFor("JSON Fragmenter"), stored) as Record<
      string,
      unknown
    >;
    expect(migrated).toBeDefined();
    expect(migrated.inputSource).toBe("attribute");
    expect(migrated.jsonAttribute).toBe("payload");
  });

  it("keeps the entries of a map at the root", () => {
    const schema = {
      type: "object",
      additionalProperties: { type: "string" },
    };
    expect(migrate(schema, { a: "one", b: "two" })).toEqual({
      a: "one",
      b: "two",
    });
  });

  it("drops a stored value no variant of the new union accepts", () => {
    const schema = {
      oneOf: [
        {
          type: "object",
          required: ["type"],
          properties: { type: { type: "string", enum: ["a"] } },
        },
        {
          type: "object",
          required: ["type"],
          properties: { type: { type: "string", enum: ["b"] } },
        },
      ],
    };
    expect(migrate(schema, { type: "a" })).toEqual({ type: "a" });
    expect(migrate(schema, { type: "gone" })).toBeUndefined();
  });
});

describe("migrateValue", () => {
  const schema = {
    type: "object",
    properties: {
      kept: { type: "string" },
      retyped: { type: "number" },
      choice: { type: "string", enum: ["a", "b"] },
      rows: {
        type: "array",
        items: {
          type: "object",
          properties: { name: { type: "string" } },
        },
      },
    },
  };

  it("keeps a field the new schema still has", () => {
    expect(migrate(schema, { kept: "x" })).toEqual({ kept: "x" });
  });

  it("drops a field the new schema no longer has", () => {
    expect(migrate(schema, { kept: "x", removed: "y" })).toEqual({ kept: "x" });
  });

  it("drops a value whose type the new schema changed", () => {
    expect(migrate(schema, { retyped: "not a number" })).toEqual({});
  });

  it("drops an enum value the new schema no longer offers", () => {
    expect(migrate(schema, { choice: "c" })).toEqual({});
    expect(migrate(schema, { choice: "b" })).toEqual({ choice: "b" });
  });

  it("prunes inside array rows rather than dropping the array", () => {
    expect(
      migrate(schema, { rows: [{ name: "a", gone: 1 }, { name: "b" }] }),
    ).toEqual({ rows: [{ name: "a" }, { name: "b" }] });
  });

  it("keeps an expression field and normalises a type it no longer allows", () => {
    const codeSchema = {
      type: "object",
      properties: {
        expr: {
          type: "object",
          format: "code",
          required: ["type", "value"],
          properties: {
            type: { type: "string", enum: ["flowExpr"] },
            value: { type: "string" },
          },
        },
      },
    };
    expect(
      migrate(codeSchema, { expr: { type: "string", value: "a" } }),
    ).toEqual({ expr: { type: "flowExpr", value: "a" } });
  });

  it("keeps a null only where the new schema still allows one", () => {
    expect(
      migrate(
        { type: "object", properties: { a: { type: ["string", "null"] } } },
        { a: null },
      ),
    ).toEqual({ a: null });
    expect(
      migrate(
        { type: "object", properties: { a: { type: "string" } } },
        { a: null },
      ),
    ).toEqual({});
  });

  it("carries nothing the new schema rejects, for every action", () => {
    // Migrating one action's params into another's schema is the worst case a
    // rename can produce. The property is that migration never makes things
    // worse: whatever survives must not add an error that an empty value would
    // not also have — a required field still unfilled, or a root union with no
    // variant chosen, is the user's to resolve either way.
    const withParams = actions.filter((action) => action.parameter);
    const failures: string[] = [];

    for (let index = 0; index < withParams.length; index++) {
      const from = withParams[index];
      const to = withParams[(index + 1) % withParams.length];
      const stored = {
        ...((from.parameter as { properties?: object }).properties ?? {}),
        some: "value",
        type: "csv",
        conditions: [{}],
      };

      const before = validate(to.parameter as never, {});
      const migrated = migrateValue(
        compile(to.parameter as never, { actionName: to.name }),
        stored,
      );
      const after = validate(to.parameter as never, migrated ?? {});

      const introduced = Object.entries(after).filter(
        ([key, messages]) =>
          before[key] === undefined ||
          messages.some((message) => !before[key].includes(message)),
      );
      if (introduced.length > 0) {
        failures.push(
          `${from.name} -> ${to.name}: ${JSON.stringify(introduced)}`,
        );
      }
    }

    expect(failures).toEqual([]);
  });
});
