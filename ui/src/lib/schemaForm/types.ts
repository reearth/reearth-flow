/**
 * The form model the params editor renders.
 *
 * Action parameter schemas are not arbitrary JSON Schema — they are the narrow
 * dialect Rust's `schemars` emits, measured at roughly eight recognisable
 * shapes across the whole corpus. Rather than rewrite those shapes until a
 * general-purpose form engine happens to guess right (see
 * `docs/schema-form.md`), we classify each node once, up front, into one of the
 * `FieldNode` kinds below and render that.
 *
 * The two properties worth protecting as this grows:
 *
 * - **Nullability is carried, never dropped.** `nullable` says whether "unset"
 *   is a legal value for a field. The old patch layer deleted the `null` branch
 *   of `anyOf: [X, null]`, which made "unset" unrepresentable and let defaults
 *   invent values the schema never had.
 * - **Validation runs against the original schema**, so the form's verdict and
 *   the engine's verdict cannot drift apart. Nothing here is a substitute for
 *   that; it only decides what to draw.
 */
import type { JSONSchema7 } from "json-schema";

/**
 * A schema as the engine publishes it, or as a custom action supplies it.
 *
 * Draft-07. Producers are free to add keywords of their own (`format: "code"`
 * is one); the compiler reads what it recognises and reports the rest as an
 * `unsupported` field rather than failing.
 */
export type FlowSchema = JSONSchema7;

/** Location of a node in the form data. Numbers index arrays. */
export type FieldPath = readonly (string | number)[];

export type EnumOption = {
  value: unknown;
  label: string;
  description?: string;
};

/** Common to every node. */
type FieldBase = {
  path: FieldPath;
  /** Property name this node sits under; "" at the root. */
  name: string;
  title?: string;
  description?: string;
  /** True when the schema admits `null` (or omission) as a value. */
  nullable: boolean;
  /** Present only when the schema states one. Never inferred. */
  default?: unknown;
  /** The pre-normalisation schema, kept for validation display and fallback. */
  raw: JSONSchema7;
};

export type StringField = FieldBase & {
  kind: "string";
  format?: string;
  minLength?: number;
  maxLength?: number;
  pattern?: string;
};

export type NumberField = FieldBase & {
  kind: "number";
  integer: boolean;
  minimum?: number;
  maximum?: number;
  /** schemars width hint (`uint8`, `int64`, `double`, …). */
  numericFormat?: string;
};

export type BooleanField = FieldBase & { kind: "boolean" };

export type EnumField = FieldBase & {
  kind: "enum";
  options: EnumOption[];
};

export type ArrayField = FieldBase & {
  kind: "array";
  item: FieldNode;
  minItems?: number;
  maxItems?: number;
};

export type ObjectField = FieldBase & {
  kind: "object";
  properties: FieldNode[];
  required: string[];
};

/** `additionalProperties: <schema>` — a map of user-named keys. */
export type MapField = FieldBase & {
  kind: "map";
  value: FieldNode;
};

export type UnionVariant = {
  key: string;
  title: string;
  description?: string;
  /**
   * Set when every branch agrees on a required single-value enum property.
   * The property is then implied by the variant choice and is not rendered.
   */
  discriminator?: { property: string; value: unknown };
  /** Required property names, used to match untagged variants. */
  requiredKeys: string[];
  /**
   * A variant carrying no fields of its own, whose value is the bare constant
   * (`"euclidean"`) rather than an object.
   */
  constant?: unknown;
  node: FieldNode;
};

export type UnionField = FieldBase & {
  kind: "union";
  variants: UnionVariant[];
  /** True when all variants share a discriminator property. */
  tagged: boolean;
};

/** A FlowExpr / Python / literal value, i.e. `format: "code"`. */
export type ExprField = FieldBase & {
  kind: "expr";
  /** Which `type` values the schema admits. */
  allowedTypes: ("flowExpr" | "string")[];
  /** Which editor to open. `python` comes from an override, not the schema. */
  flavor: "flowExpr" | "python";
};

export type ColorField = FieldBase & { kind: "color" };
export type WysiwygField = FieldBase & { kind: "wysiwyg" };

/**
 * Anything the compiler does not recognise. Rendered as a labelled raw-JSON
 * editor rather than silently omitted — a blank field loses data, and
 * user-authored custom action schemas need not follow the schemars dialect.
 */
export type UnsupportedField = FieldBase & {
  kind: "unsupported";
  reason: string;
};

export type FieldNode =
  | StringField
  | NumberField
  | BooleanField
  | EnumField
  | ArrayField
  | ObjectField
  | MapField
  | UnionField
  | ExprField
  | ColorField
  | WysiwygField
  | UnsupportedField;

export type FieldKind = FieldNode["kind"];
