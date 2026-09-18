/**
 * Stage 1: fold a schemars-shaped JSON Schema into a uniform tree.
 *
 * Three things happen here, and nothing else:
 *
 * 1. `$ref` is resolved against `definitions`, with sibling keywords kept.
 * 2. `allOf` is merged into its parent — schemars wraps a `$ref` in a
 *    single-element `allOf` whenever it also has a `title` or `default`, which
 *    accounts for every `allOf` in the corpus.
 * 3. Nullability is **lifted, not deleted**: `type: [X, "null"]` and
 *    `anyOf: [X, {type: "null"}]` both become `nullable: true` on a node that
 *    still describes `X`.
 *
 * The walk is uniform: it descends `properties`, `items`, `additionalProperties`
 * *and every remaining `oneOf`/`anyOf` branch*. The old patch layer had four
 * passes with three different traversals, none of which entered a `oneOf`
 * branch, which is why 13 sites across 5 actions still had an unresolved
 * `allOf: [{$ref}]` sitting inside one.
 *
 * `oneOf` is deliberately left standing. It carries meaning — a union — and
 * deciding what that union looks like is `compile`'s job, not this one's.
 */
import type { JSONSchema7, JSONSchema7Definition } from "json-schema";

export type JsonType =
  | "string"
  | "number"
  | "integer"
  | "boolean"
  | "object"
  | "array";

export type NormalizedNode = {
  type?: JsonType;
  /** Set when the schema lists several non-null types, e.g. `[boolean, number, string]`. */
  types?: JsonType[];
  nullable: boolean;
  title?: string;
  description?: string;
  default?: unknown;
  format?: string;
  enum?: unknown[];
  const?: unknown;
  properties?: Record<string, NormalizedNode>;
  required: string[];
  items?: NormalizedNode;
  additionalProperties?: false | NormalizedNode;
  oneOf?: NormalizedNode[];
  /** Residual `anyOf` after null branches are lifted out. */
  anyOf?: NormalizedNode[];
  minimum?: number;
  maximum?: number;
  minItems?: number;
  maxItems?: number;
  minLength?: number;
  maxLength?: number;
  pattern?: string;
  /** True when a `$ref` cycle was cut here. */
  recursive?: boolean;
  /** The schema as it arrived, before any of the above. */
  raw: JSONSchema7;
};

const DEF_PREFIX = "#/definitions/";
const ALT_DEF_PREFIX = "#/$defs/";

const isSchema = (value: JSONSchema7Definition): value is JSONSchema7 =>
  typeof value !== "boolean";

const isNullBranch = (value: JSONSchema7Definition): boolean =>
  isSchema(value) &&
  value.type === "null" &&
  value.properties === undefined &&
  value.$ref === undefined;

const refName = (ref: string): string | null => {
  if (ref.startsWith(DEF_PREFIX)) return ref.slice(DEF_PREFIX.length);
  if (ref.startsWith(ALT_DEF_PREFIX)) return ref.slice(ALT_DEF_PREFIX.length);
  return null;
};

/**
 * Merge `source` beneath `target`: keywords already on `target` win, because a
 * schemars wrapper carries the caller's `title`/`description`/`default` and the
 * referent carries the shape.
 */
const mergeUnder = (target: JSONSchema7, source: JSONSchema7): JSONSchema7 => {
  const merged: JSONSchema7 = { ...source, ...target };

  // `required` is additive rather than overridden.
  if (source.required || target.required) {
    merged.required = [
      ...new Set([...(source.required ?? []), ...(target.required ?? [])]),
    ];
  }
  // Properties from both sides coexist; the target's definition of a shared
  // key wins, matching the keyword rule above.
  if (source.properties || target.properties) {
    merged.properties = { ...source.properties, ...target.properties };
  }
  delete merged.$ref;
  delete merged.allOf;
  return merged;
};

/**
 * Collapse `$ref` and `allOf` until neither remains. Returns the flattened
 * schema plus whether a reference cycle was cut.
 */
const flatten = (
  schema: JSONSchema7,
  definitions: Record<string, JSONSchema7Definition>,
  seen: ReadonlySet<string>,
): { schema: JSONSchema7; seen: Set<string>; recursive: boolean } => {
  let current = schema;
  const visited = new Set(seen);
  let recursive = false;

  // Bounded by the number of definitions; schemars nests these only a level or
  // two deep, but the guard keeps a malformed custom schema from spinning.
  for (let guard = 0; guard < 100; guard++) {
    if (typeof current.$ref === "string") {
      const name = refName(current.$ref);
      const target = name ? definitions[name] : undefined;
      if (!name || target === undefined || !isSchema(target)) break;
      if (visited.has(name)) {
        recursive = true;
        const { $ref: _dropped, ...rest } = current;
        current = rest;
        continue;
      }
      visited.add(name);
      current = mergeUnder(current, target);
      continue;
    }

    if (Array.isArray(current.allOf) && current.allOf.length > 0) {
      const branches = current.allOf.filter(isSchema);
      const { allOf: _dropped, ...rest } = current;
      let merged: JSONSchema7 = rest;
      for (const branch of branches) {
        const flat = flatten(branch, definitions, visited);
        for (const name of flat.seen) visited.add(name);
        recursive = recursive || flat.recursive;
        merged = mergeUnder(merged, flat.schema);
      }
      current = merged;
      continue;
    }

    break;
  }

  return { schema: current, seen: visited, recursive };
};

/** Split a `type` keyword into its non-null types and a nullability flag. */
const splitType = (
  type: JSONSchema7["type"],
): { types: JsonType[]; nullable: boolean } => {
  const list = Array.isArray(type) ? type : type ? [type] : [];
  return {
    types: list.filter((entry): entry is JsonType => entry !== "null"),
    nullable: list.includes("null"),
  };
};

export const normalize = (
  schema: JSONSchema7Definition | null | undefined,
  definitions?: Record<string, JSONSchema7Definition>,
): NormalizedNode => {
  // 30 of the engine's actions take no parameters at all and publish
  // `parameter: null`; callers hand that straight through.
  if (schema === null || schema === undefined) {
    return { nullable: true, required: [], raw: {} };
  }
  const defs =
    definitions ??
    ((isSchema(schema) ? schema.definitions : undefined) as
      | Record<string, JSONSchema7Definition>
      | undefined) ??
    {};
  return normalizeNode(schema, defs, new Set());
};

const normalizeNode = (
  input: JSONSchema7Definition,
  defs: Record<string, JSONSchema7Definition>,
  seen: ReadonlySet<string>,
): NormalizedNode => {
  // `true`/`false` in schema position: `true` admits anything, `false` nothing.
  if (!isSchema(input)) {
    return { nullable: input, required: [], raw: {} };
  }

  const flat = flatten(input, defs, seen);
  const schema = flat.schema;
  const descend = (child: JSONSchema7Definition) =>
    normalizeNode(child, defs, flat.seen);

  const { types, nullable: nullableFromType } = splitType(schema.type);
  let nullable = nullableFromType;

  const node: NormalizedNode = {
    nullable: false,
    required: schema.required ? [...schema.required] : [],
    raw: input as JSONSchema7,
  };

  if (schema.title !== undefined) node.title = schema.title;
  if (schema.description !== undefined) node.description = schema.description;
  if (schema.format !== undefined) node.format = schema.format;
  if (schema.const !== undefined) node.const = schema.const;
  if (Array.isArray(schema.enum)) {
    // `enum: [..., null]` is another way of spelling nullable.
    const values = schema.enum.filter((value) => value !== null);
    if (values.length !== schema.enum.length) nullable = true;
    node.enum = values;
  }
  if ("default" in schema) node.default = schema.default;

  if (typeof schema.minimum === "number") node.minimum = schema.minimum;
  if (typeof schema.maximum === "number") node.maximum = schema.maximum;
  if (typeof schema.minItems === "number") node.minItems = schema.minItems;
  if (typeof schema.maxItems === "number") node.maxItems = schema.maxItems;
  if (typeof schema.minLength === "number") node.minLength = schema.minLength;
  if (typeof schema.maxLength === "number") node.maxLength = schema.maxLength;
  if (typeof schema.pattern === "string") node.pattern = schema.pattern;

  // --- combinators -----------------------------------------------------
  // A null branch anywhere means the field may be unset. The remaining
  // branches keep describing the value; if exactly one survives, it merges
  // into this node so `anyOf: [{$ref: X}, null]` reads as "nullable X"
  // without X's identity being thrown away.
  const liftNull = (
    branches: JSONSchema7Definition[],
  ): JSONSchema7Definition[] => {
    const kept = branches.filter((branch) => !isNullBranch(branch));
    if (kept.length !== branches.length) nullable = true;
    return kept;
  };

  let oneOf = Array.isArray(schema.oneOf) ? liftNull(schema.oneOf) : undefined;
  let anyOf = Array.isArray(schema.anyOf) ? liftNull(schema.anyOf) : undefined;

  // A sole surviving branch is not a choice — fold it in, keeping this node's
  // own title/description/default on top.
  const fold = (branches: JSONSchema7Definition[] | undefined) => {
    if (!branches || branches.length !== 1) return branches;
    const only = branches[0];
    if (!isSchema(only)) return branches;
    const inner = normalizeNode(only, defs, flat.seen);
    nullable = nullable || inner.nullable;
    if (node.title === undefined && inner.title !== undefined)
      node.title = inner.title;
    if (node.description === undefined && inner.description !== undefined)
      node.description = inner.description;
    if (node.format === undefined && inner.format !== undefined)
      node.format = inner.format;
    if (node.enum === undefined && inner.enum !== undefined)
      node.enum = inner.enum;
    if (node.const === undefined && inner.const !== undefined)
      node.const = inner.const;
    if (node.default === undefined && inner.default !== undefined)
      node.default = inner.default;
    if (inner.type && types.length === 0) types.push(inner.type);
    if (inner.properties) node.properties = inner.properties;
    if (inner.required.length) {
      node.required = [...new Set([...node.required, ...inner.required])];
    }
    if (inner.items) node.items = inner.items;
    if (inner.additionalProperties !== undefined)
      node.additionalProperties = inner.additionalProperties;
    // A folded branch may itself be a choice (`anyOf: [{$ref: X}, null]` where
    // X is a union of scalars). Carry it up rather than losing it.
    if (inner.oneOf) node.oneOf = inner.oneOf;
    if (inner.anyOf) node.anyOf = inner.anyOf;
    if (inner.minimum !== undefined && node.minimum === undefined)
      node.minimum = inner.minimum;
    if (inner.maximum !== undefined && node.maximum === undefined)
      node.maximum = inner.maximum;
    if (inner.recursive) node.recursive = true;
    return undefined;
  };

  oneOf = fold(oneOf);
  anyOf = fold(anyOf);

  if (oneOf && oneOf.length > 0) node.oneOf = oneOf.map(descend);
  if (anyOf && anyOf.length > 0) node.anyOf = anyOf.map(descend);

  // --- structure -------------------------------------------------------
  if (schema.properties) {
    node.properties = {
      ...node.properties,
      ...Object.fromEntries(
        Object.entries(schema.properties).map(([key, value]) => [
          key,
          descend(value),
        ]),
      ),
    };
  }

  if (schema.items !== undefined) {
    // Tuple form does not occur in the corpus; the first entry is the closest
    // honest reading, and `compile` flags the rest as unsupported.
    node.items = Array.isArray(schema.items)
      ? schema.items[0] !== undefined
        ? descend(schema.items[0])
        : undefined
      : descend(schema.items);
  }

  if (schema.additionalProperties !== undefined) {
    node.additionalProperties =
      schema.additionalProperties === false
        ? false
        : schema.additionalProperties === true
          ? undefined
          : descend(schema.additionalProperties);
  }

  if (types.length === 1) node.type = types[0];
  else if (types.length > 1) node.types = types;

  node.nullable = nullable;
  if (flat.recursive) node.recursive = true;

  return node;
};
