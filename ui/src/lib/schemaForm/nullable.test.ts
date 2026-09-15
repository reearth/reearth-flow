/**
 * The nullable-enum bug the compiler exists to make impossible.
 *
 * Every case here is one the previous patch-then-RJSF pipeline got wrong; the
 * assertions record what the engine's schema actually says, so a regression
 * shows up as a disagreement with the engine rather than as a UI complaint.
 */
import { describe, expect, it } from "vitest";

import { compile, selectVariant } from "./compile";
import { normalize } from "./normalize";
import type { EnumField, FieldNode, ObjectField, UnionField } from "./types";

const nullableEnum = {
  type: "object",
  properties: {
    encoding: {
      title: "Encoding",
      anyOf: [{ $ref: "#/definitions/Enc" }, { type: "null" }],
    },
  },
  definitions: {
    Enc: {
      oneOf: [
        { title: "Text", type: "string", enum: ["text"] },
        { title: "Base64", type: "string", enum: ["base64"] },
      ],
    },
  },
} as unknown as Parameters<typeof compile>[0];

const propertyOf = (
  root: ReturnType<typeof compile>,
  name: string,
): FieldNode => {
  expect(root.kind).toBe("object");
  const found = (root as ObjectField).properties.find((p) => p.name === name);
  if (!found) throw new Error(`no field named ${name}`);
  return found;
};

describe("nullable enum", () => {
  it("keeps the enum and records that it may be unset", () => {
    const encoding = propertyOf(compile(nullableEnum), "encoding");

    expect(encoding.kind).toBe("enum");
    expect(encoding.nullable).toBe(true);
    expect((encoding as EnumField).options).toEqual([
      { value: "text", label: "Text" },
      { value: "base64", label: "Base64" },
    ]);
  });

  it("keeps the caller's title rather than the referent's", () => {
    expect(propertyOf(compile(nullableEnum), "encoding").title).toBe(
      "Encoding",
    );
  });

  it("invents no default where the schema states none", () => {
    // The old pipeline turned this field into a non-null `oneOf` of consts,
    // which made RJSF materialise option 0 ("text") on mount and write it back
    // — silently disabling HTTP Caller's content-type sniffing.
    expect(
      propertyOf(compile(nullableEnum), "encoding").default,
    ).toBeUndefined();
  });

  it("lifts nullability out of a type array too", () => {
    const node = normalize({
      type: ["string", "null"],
    } as never);
    expect(node.type).toBe("string");
    expect(node.nullable).toBe(true);
  });

  it("lifts nullability out of an enum containing null", () => {
    const node = normalize({ enum: ["a", "b", null] } as never);
    expect(node.enum).toEqual(["a", "b"]);
    expect(node.nullable).toBe(true);
  });

  it("marks a nullable object nullable without making it required", () => {
    const root = compile({
      type: "object",
      properties: {
        retry: {
          title: "Retry",
          anyOf: [{ $ref: "#/definitions/Retry" }, { type: "null" }],
        },
      },
      definitions: {
        Retry: {
          type: "object",
          properties: { attempts: { type: "integer" } },
        },
      },
    } as never);
    const retry = propertyOf(root, "retry");
    expect(retry.kind).toBe("object");
    expect(retry.nullable).toBe(true);
    expect((retry as ObjectField).properties.map((p) => p.name)).toEqual([
      "attempts",
    ]);
  });
});

describe("allOf inside a oneOf branch", () => {
  // 13 sites across 5 actions kept an unresolved `allOf: [{$ref}]` here,
  // because none of the old passes descended into a `oneOf` branch.
  it("resolves the ref", () => {
    const root = compile({
      type: "object",
      properties: { mode: { allOf: [{ $ref: "#/definitions/Mode" }] } },
      definitions: {
        Mode: {
          oneOf: [
            {
              type: "object",
              required: ["type"],
              properties: {
                type: { type: "string", enum: ["all"] },
                listName: {
                  title: "List Name",
                  default: "_indices",
                  allOf: [{ $ref: "#/definitions/Attribute" }],
                },
              },
            },
            {
              type: "object",
              required: ["type"],
              properties: {
                type: { type: "string", enum: ["one"] },
                index: { type: "integer" },
              },
            },
          ],
        },
        Attribute: { type: "string" },
      },
    } as never);

    const mode = propertyOf(root, "mode") as UnionField;
    expect(mode.kind).toBe("union");
    expect(mode.tagged).toBe(true);

    const first = mode.variants[0].node as ObjectField;
    const listName = first.properties.find((p) => p.name === "listName");
    expect(listName?.kind).toBe("string");
    expect(listName?.title).toBe("List Name");
    expect(listName?.default).toBe("_indices");
  });

  it("hides the discriminator, which the old form rendered as a second dropdown", () => {
    const root = compile({
      type: "object",
      properties: { mode: { $ref: "#/definitions/Mode" } },
      definitions: {
        Mode: {
          oneOf: [
            {
              title: "All",
              type: "object",
              required: ["type"],
              properties: { type: { type: "string", enum: ["all"] } },
            },
            {
              title: "One",
              type: "object",
              required: ["type", "index"],
              properties: {
                type: { type: "string", enum: ["one"] },
                index: { type: "integer" },
              },
            },
          ],
        },
      },
    } as never);

    const mode = propertyOf(root, "mode") as UnionField;
    expect(mode.variants.map((v) => v.title)).toEqual(["All", "One"]);
    expect(mode.variants[0].discriminator).toEqual({
      property: "type",
      value: "all",
    });
    expect((mode.variants[0].node as ObjectField).properties).toEqual([]);
    expect(
      (mode.variants[1].node as ObjectField).properties.map((p) => p.name),
    ).toEqual(["index"]);
  });
});

describe("selectVariant", () => {
  const union = propertyOf(
    compile({
      type: "object",
      properties: { auth: { $ref: "#/definitions/Auth" } },
      definitions: {
        Auth: {
          oneOf: [
            {
              title: "Basic",
              type: "object",
              required: ["type", "user"],
              properties: {
                type: { type: "string", enum: ["basic"] },
                user: { type: "string" },
              },
            },
            {
              title: "Bearer",
              type: "object",
              required: ["type", "token"],
              properties: {
                type: { type: "string", enum: ["bearer"] },
                token: { type: "string" },
              },
            },
          ],
        },
      },
    } as never),
    "auth",
  ) as UnionField;

  it("reads the tag instead of scoring branches", () => {
    expect(selectVariant(union, { type: "bearer", token: "x" })).toBe(1);
    expect(selectVariant(union, { type: "basic", user: "x" })).toBe(0);
  });

  it("reports no variant for an unset value rather than guessing one", () => {
    expect(selectVariant(union, null)).toBe(-1);
    expect(selectVariant(union, undefined)).toBe(-1);
  });

  it("matches untagged variants by their required keys", () => {
    const geometry = propertyOf(
      compile({
        type: "object",
        properties: { geometry: { $ref: "#/definitions/G" } },
        definitions: {
          G: {
            oneOf: [
              {
                title: "WKT",
                type: "object",
                required: ["column"],
                properties: { column: { type: "string" } },
              },
              {
                title: "Coordinates",
                type: "object",
                required: ["xColumn", "yColumn"],
                properties: {
                  xColumn: { type: "string" },
                  yColumn: { type: "string" },
                },
              },
            ],
          },
        },
      } as never),
      "geometry",
    ) as UnionField;

    expect(geometry.tagged).toBe(false);
    expect(selectVariant(geometry, { column: "geom" })).toBe(0);
    expect(selectVariant(geometry, { xColumn: "x", yColumn: "y" })).toBe(1);
  });

  it("matches a scalar union on the value's own type", () => {
    const mapped = propertyOf(
      compile({
        type: "object",
        properties: {
          value: {
            anyOf: [
              { title: "Text", type: "string" },
              { title: "Number", type: "number" },
              { title: "True or False", type: "boolean" },
            ],
          },
        },
      } as never),
      "value",
    ) as UnionField;

    expect(mapped.kind).toBe("union");
    expect(selectVariant(mapped, "3")).toBe(0);
    expect(selectVariant(mapped, 3)).toBe(1);
    expect(selectVariant(mapped, true)).toBe(2);
  });
});
