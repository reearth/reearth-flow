/**
 * The story fixtures compile to the kinds their stories are named after.
 *
 * A fixture that stops producing, say, a `map` is not a visible failure — the
 * story still renders, just not the thing it exists to show. `corpus.test.ts`
 * does the same job for the engine's real schemas; this does it for the
 * hand-written ones.
 */
import { describe, expect, it } from "vitest";

import { compile } from "@flow/lib/schemaForm";
import type { FieldKind, FieldNode } from "@flow/lib/schemaForm";

import {
  CONTAINERS_SCHEMA,
  FIELD_KINDS_SCHEMA,
  NULLABILITY_SCHEMA,
  PYTHON_SCRIPT_SCHEMA,
  UNIONS_SCHEMA,
  UNSUPPORTED_SCHEMA,
  VALIDATION_SCHEMA,
} from "./storyFixtures";

/**
 * Every kind in the tree.
 *
 * A union's variants share their parent's path — the variant is a choice about
 * one field, not a field of its own — so they are collected here but not keyed
 * by path below.
 */
const allKinds = (node: FieldNode, out: FieldKind[] = []): FieldKind[] => {
  out.push(node.kind);
  if (node.kind === "object") node.properties.forEach((c) => allKinds(c, out));
  if (node.kind === "array") allKinds(node.item, out);
  if (node.kind === "map") allKinds(node.value, out);
  if (node.kind === "union")
    node.variants.forEach((v) => allKinds(v.node, out));
  return out;
};

/** Each property of an object root, by name. Unions are not descended into. */
const kindByName = (schema: Parameters<typeof compile>[0]) => {
  const root = compile(schema);
  if (root.kind !== "object") throw new Error("expected an object root");
  return Object.fromEntries(root.properties.map((p) => [p.name, p.kind]));
};

const kinds = (schema: Parameters<typeof compile>[0]) =>
  new Set(allKinds(compile(schema)));

describe("story fixtures", () => {
  it("covers every field kind in one schema", () => {
    // The gallery story is the one place the whole renderer is visible at once,
    // so a new kind has to appear here or it has no story at all.
    const every: FieldKind[] = [
      "string",
      "number",
      "boolean",
      "enum",
      "array",
      "object",
      "map",
      "union",
      "expr",
      "color",
      "wysiwyg",
      "unsupported",
    ];
    expect([...kinds(FIELD_KINDS_SCHEMA)].sort()).toEqual([...every].sort());
  });

  it("maps each field-kind property to its kind", () => {
    expect(kindByName(FIELD_KINDS_SCHEMA)).toEqual({
      name: "string",
      featureCount: "number",
      tolerance: "number",
      keepOriginal: "boolean",
      mode: "enum",
      groupBy: "array",
      extent: "object",
      renames: "map",
      projection: "union",
      filter: "expr",
      strokeColor: "color",
      notes: "wysiwyg",
      legacyOption: "unsupported",
    });
  });

  it("carries nullability rather than dropping it", () => {
    const root = compile(NULLABILITY_SCHEMA);
    if (root.kind !== "object") throw new Error("expected an object root");
    expect(
      Object.fromEntries(root.properties.map((f) => [f.name, f.nullable])),
    ).toEqual({
      attribute: true,
      limit: true,
      caseSensitive: true,
      bounds: true,
      tags: true,
    });
  });

  it("draws unions, tagged and untagged", () => {
    const root = compile(UNIONS_SCHEMA);
    if (root.kind !== "object") throw new Error("expected an object root");
    const union = (name: string) => {
      const node = root.properties.find((p) => p.name === name);
      if (node?.kind !== "union") throw new Error(`${name} is not a union`);
      return node;
    };
    expect(union("projection").tagged).toBe(true);
    expect(union("projection").variants.map((v) => v.key)).toEqual([
      "epsg",
      "wkt",
      "passthrough",
    ]);
    expect(union("selector").tagged).toBe(false);
  });

  it("recognises every shape in the fixtures that has a control", () => {
    // UNSUPPORTED_SCHEMA is the deliberate exception: it exists to show the
    // raw-JSON fallback.
    const fixtures = {
      validation: VALIDATION_SCHEMA,
      nullability: NULLABILITY_SCHEMA,
      unions: UNIONS_SCHEMA,
      containers: CONTAINERS_SCHEMA,
      pythonScript: PYTHON_SCRIPT_SCHEMA,
      unsupported: UNSUPPORTED_SCHEMA,
    };
    expect(
      Object.fromEntries(
        Object.entries(fixtures).map(([name, schema]) => [
          name,
          kinds(schema).has("unsupported"),
        ]),
      ),
    ).toEqual({
      validation: false,
      nullability: false,
      unions: false,
      containers: false,
      pythonScript: false,
      unsupported: true,
    });
  });
});
