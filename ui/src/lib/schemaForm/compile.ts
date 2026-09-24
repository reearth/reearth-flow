/**
 * Stage 2: classify a normalised tree into the field kinds the renderer draws.
 *
 * Everything the renderer needs is decided once, here, from the schema's own
 * shape — not rediscovered per-field at render time. In particular an
 * expression field is recognised by `format: "code"` on the spot, so there is
 * no second, unpatched copy of the schema to consult and no scan of every
 * definition for a property that happens to share a name.
 *
 * Anything unrecognised becomes an `unsupported` node rather than nothing at
 * all. `corpus.test.ts` asserts the built-in actions produce none.
 */
import type { JSONSchema7Definition } from "json-schema";

import { normalize } from "./normalize";
import type { NormalizedNode } from "./normalize";
import { childPath, pathKey } from "./path";
import type {
  EnumOption,
  FieldNode,
  FieldPath,
  UnionField,
  UnionVariant,
} from "./types";

export type CompileOptions = {
  /**
   * Which expression fields open the Python editor instead of the FlowExpr
   * one, by action name and dot path. The distinction is not in the schema —
   * both are `format: "code"` — so it is declared here rather than inferred
   * inside a widget from the field's name.
   */
  pythonFields?: { actionName: string; path: string }[];
  actionName?: string;
};

const DEFAULT_PYTHON_FIELDS = [
  { actionName: "Python Script Processor", path: "script" },
];

type Ctx = {
  path: FieldPath;
  name: string;
  options: CompileOptions;
};

/** A branch that is just one literal value: `{const: x}` or `{enum: [x]}`. */
const constantValue = (node: NormalizedNode): unknown | undefined => {
  if (node.const !== undefined) return node.const;
  if (node.enum?.length === 1) return node.enum[0];
  return undefined;
};

const isConstantOnly = (node: NormalizedNode): boolean =>
  constantValue(node) !== undefined &&
  node.properties === undefined &&
  node.items === undefined &&
  node.oneOf === undefined;

const labelFor = (node: NormalizedNode, value: unknown): string =>
  node.title ?? String(value);

/**
 * The property that identifies which variant an object is, when schemars has
 * emitted an internally tagged enum: a required property pinned to one value.
 */
const discriminatorOf = (
  node: NormalizedNode,
): { property: string; value: unknown } | undefined => {
  if (!node.properties) return undefined;
  for (const [key, child] of Object.entries(node.properties)) {
    if (!node.required.includes(key)) continue;
    const value = constantValue(child);
    if (value !== undefined) return { property: key, value };
  }
  return undefined;
};

const enumOptionsFromBranches = (
  branches: NormalizedNode[],
): EnumOption[] | undefined => {
  const options: EnumOption[] = [];
  for (const branch of branches) {
    const value = constantValue(branch);
    if (value === undefined || !isConstantOnly(branch)) return undefined;
    options.push({
      value,
      label: labelFor(branch, value),
      ...(branch.description ? { description: branch.description } : {}),
    });
  }
  return options;
};

const base = (node: NormalizedNode, ctx: Ctx) => ({
  path: ctx.path,
  name: ctx.name,
  ...(node.title !== undefined ? { title: node.title } : {}),
  ...(node.description !== undefined ? { description: node.description } : {}),
  nullable: node.nullable,
  ...(node.default !== undefined ? { default: node.default } : {}),
  raw: node.raw,
});

const unsupported = (
  node: NormalizedNode,
  ctx: Ctx,
  reason: string,
): FieldNode => ({ ...base(node, ctx), kind: "unsupported", reason });

const buildVariants = (
  branches: NormalizedNode[],
  ctx: Ctx,
): { variants: UnionVariant[]; tagged: boolean } => {
  const discriminators = branches.map((branch) =>
    branch.properties ? discriminatorOf(branch) : undefined,
  );
  const objectIndices = branches
    .map((branch, index) => (branch.properties ? index : -1))
    .filter((index) => index >= 0);

  const names = new Set(
    objectIndices.map((index) => discriminators[index]?.property),
  );
  const tagged =
    objectIndices.length > 0 &&
    names.size === 1 &&
    !names.has(undefined) &&
    new Set(objectIndices.map((index) => discriminators[index]?.value)).size ===
      objectIndices.length;

  const variants = branches.map((branch, index): UnionVariant => {
    const discriminator = tagged ? discriminators[index] : undefined;
    const constant = constantValue(branch);
    const isUnit = isConstantOnly(branch);

    // The tag is implied by the variant choice, so it is not a field the user
    // fills in. Dropping it here is what removes the redundant second dropdown
    // the old form rendered under every union.
    const visible: NormalizedNode = discriminator
      ? {
          ...branch,
          properties: Object.fromEntries(
            Object.entries(branch.properties ?? {}).filter(
              ([key]) => key !== discriminator.property,
            ),
          ),
          required: branch.required.filter(
            (key) => key !== discriminator.property,
          ),
        }
      : branch;

    const key = discriminator
      ? String(discriminator.value)
      : isUnit
        ? String(constant)
        : (branch.title ?? `variant${index}`);

    return {
      key,
      title: branch.title ?? key,
      ...(branch.description ? { description: branch.description } : {}),
      ...(discriminator ? { discriminator } : {}),
      requiredKeys: visible.required,
      ...(isUnit ? { constant } : {}),
      node: compileNode(visible, { ...ctx, name: key }),
    };
  });

  return { variants, tagged };
};

const compileNode = (node: NormalizedNode, ctx: Ctx): FieldNode => {
  const common = base(node, ctx);

  // --- expression / code ------------------------------------------------
  if (node.format === "code") {
    const allowed = node.properties?.type?.enum;
    const allowedTypes = (
      Array.isArray(allowed) && allowed.length > 0 ? allowed : ["flowExpr"]
    ).filter(
      (value): value is "flowExpr" | "string" =>
        value === "flowExpr" || value === "string",
    );
    const key = pathKey(ctx.path);
    const pythonFields = ctx.options.pythonFields ?? DEFAULT_PYTHON_FIELDS;
    const isPython = pythonFields.some(
      (entry) =>
        entry.actionName === ctx.options.actionName && entry.path === key,
    );
    return {
      ...common,
      kind: "expr",
      allowedTypes: allowedTypes.length ? allowedTypes : ["flowExpr"],
      flavor: isPython ? "python" : "flowExpr",
    };
  }

  if (node.format === "color") return { ...common, kind: "color" };
  if (node.format === "wysiwyg") return { ...common, kind: "wysiwyg" };

  // --- unions -----------------------------------------------------------
  // `oneOf` is a choice by definition; a residual `anyOf` of two or more
  // branches (the null branch having already been lifted out) is the untagged
  // form of the same thing.
  const branches =
    node.oneOf && node.oneOf.length > 1
      ? node.oneOf
      : node.anyOf && node.anyOf.length > 1
        ? node.anyOf
        : undefined;

  if (branches) {
    // Every branch a bare constant means this is an enum wearing a union's
    // clothes — the common schemars spelling for a plain Rust enum.
    const options = enumOptionsFromBranches(branches);
    if (options) return { ...common, kind: "enum", options };

    const { variants, tagged } = buildVariants(branches, ctx);
    return { ...common, kind: "union", variants, tagged };
  }

  // --- enums ------------------------------------------------------------
  if (node.enum && node.enum.length > 0) {
    return {
      ...common,
      kind: "enum",
      options: node.enum.map((value) => ({ value, label: String(value) })),
    };
  }

  if (node.const !== undefined) {
    return {
      ...common,
      kind: "enum",
      options: [{ value: node.const, label: String(node.const) }],
    };
  }

  // --- scalars and containers -------------------------------------------
  const type =
    node.type ??
    (node.properties || node.additionalProperties
      ? "object"
      : node.items
        ? "array"
        : undefined);

  switch (type) {
    case "string":
      return {
        ...common,
        kind: "string",
        ...(node.format ? { format: node.format } : {}),
        ...(node.minLength !== undefined ? { minLength: node.minLength } : {}),
        ...(node.maxLength !== undefined ? { maxLength: node.maxLength } : {}),
        ...(node.pattern !== undefined ? { pattern: node.pattern } : {}),
      };

    case "number":
    case "integer":
      return {
        ...common,
        kind: "number",
        integer: type === "integer",
        ...(node.minimum !== undefined ? { minimum: node.minimum } : {}),
        ...(node.maximum !== undefined ? { maximum: node.maximum } : {}),
        ...(node.format ? { numericFormat: node.format } : {}),
      };

    case "boolean":
      return { ...common, kind: "boolean" };

    case "array": {
      if (!node.items) return unsupported(node, ctx, "array without items");
      return {
        ...common,
        kind: "array",
        item: compileNode(node.items, {
          ...ctx,
          path: childPath(ctx.path, 0),
          name: ctx.name,
        }),
        ...(node.minItems !== undefined ? { minItems: node.minItems } : {}),
        ...(node.maxItems !== undefined ? { maxItems: node.maxItems } : {}),
      };
    }

    case "object": {
      const entries = Object.entries(node.properties ?? {});
      if (entries.length === 0 && node.additionalProperties) {
        return {
          ...common,
          kind: "map",
          value: compileNode(node.additionalProperties, {
            ...ctx,
            path: childPath(ctx.path, "*"),
          }),
        };
      }
      return {
        ...common,
        kind: "object",
        required: node.required,
        properties: entries.map(([key, child]) =>
          compileNode(child, {
            ...ctx,
            path: childPath(ctx.path, key),
            name: key,
          }),
        ),
      };
    }

    default:
      if (node.types?.length) {
        return unsupported(
          node,
          ctx,
          `multiple types: ${node.types.join(", ")}`,
        );
      }
      if (node.recursive) return unsupported(node, ctx, "recursive schema");
      return unsupported(node, ctx, "no type");
  }
};

export const compile = (
  schema: JSONSchema7Definition | null | undefined,
  options: CompileOptions = {},
): FieldNode => compileNode(normalize(schema), { path: [], name: "", options });

/**
 * Which variant of a union a value belongs to, or -1 for none.
 *
 * Deterministic and cheap, in place of scoring each branch by validating the
 * form data against it: the discriminator says so outright when there is one,
 * and the untagged cases fall back to the value's own shape.
 */
export const selectVariant = (field: UnionField, value: unknown): number => {
  if (value === null || value === undefined) return -1;

  // 1. Tagged: the value carries its own answer.
  if (typeof value === "object" && !Array.isArray(value)) {
    const record = value as Record<string, unknown>;
    const tagged = field.variants.findIndex(
      (variant) =>
        variant.discriminator !== undefined &&
        record[variant.discriminator.property] === variant.discriminator.value,
    );
    if (tagged >= 0) return tagged;
  }

  // 2. Unit variants carry a bare constant.
  const constant = field.variants.findIndex(
    (variant) => variant.constant !== undefined && variant.constant === value,
  );
  if (constant >= 0) return constant;

  // 3. Scalar variants match on the value's own type.
  const scalarKind =
    typeof value === "string"
      ? "string"
      : typeof value === "number"
        ? "number"
        : typeof value === "boolean"
          ? "boolean"
          : undefined;
  if (scalarKind) {
    const scalar = field.variants.findIndex(
      (variant) => variant.node.kind === scalarKind,
    );
    if (scalar >= 0) return scalar;
  }

  // 4. Untagged objects: the variant whose required keys the value satisfies,
  //    preferring the most specific match.
  if (typeof value === "object" && !Array.isArray(value)) {
    const record = value as Record<string, unknown>;
    let best = -1;
    let bestScore = -1;
    field.variants.forEach((variant, index) => {
      if (variant.requiredKeys.length === 0) return;
      const satisfied = variant.requiredKeys.every(
        (key) => record[key] !== undefined,
      );
      if (satisfied && variant.requiredKeys.length > bestScore) {
        best = index;
        bestScore = variant.requiredKeys.length;
      }
    });
    if (best >= 0) return best;
  }

  return -1;
};
