import { createSchemaUtils, RJSFSchema } from "@rjsf/utils";
import validator from "@rjsf/validator-ajv8";
import { JSONSchema7, JSONSchema7Definition } from "json-schema";

// This is a workaround for the `anyOf` type for RJSF/JSON Schema. Currently if "null" only is passed as a type in `anyof` it won't work as expected.
// We should regualry check this issue and update RJSF once a fix is published. (https://github.com/rjsf-team/react-jsonschema-form/issues/4380)
// Update March 2025: a new function was added to check if anyOf is nested in a oneOf as it will override the anyOf simplification

// Utility function to check if schema is a valid JSONSchema
const isJSONSchema = (schema: JSONSchema7Definition): schema is JSONSchema7 =>
  typeof schema !== "boolean";

// The branch standing for "this optional parameter is not set". It is titled with
// the same marker the select widget already uses for an empty value, and the
// widget drops it from the list it renders: the schema keeps a way to say
// "nothing" — without which RJSF invents a value — while the user still sees the
// one "-" entry they always had, rather than a second, invented wording for it.
export const NULL_BRANCH_TITLE = "-";
const NULL_BRANCH: JSONSchema7 = { type: "null", title: NULL_BRANCH_TITLE };

// A schema RJSF renders by choosing among sub-schemas rather than as one input.
// Collapsing `anyOf: [X, null]` to X costs nothing when X is a plain scalar, but
// for one of these it discards the only way the data can say "not set" — RJSF
// then fills the parameter in from the first branch, so an optional block the
// user never opened arrives populated and clearing it never survives a remount.
const isBranchingSchema = (schema: JSONSchema7Definition): boolean =>
  isJSONSchema(schema) &&
  (!!schema.$ref ||
    !!schema.oneOf ||
    !!schema.anyOf ||
    !!schema.allOf ||
    schema.type === "object");

// The branches of a union, if `schema` is one (directly or through a `$ref`).
// Flattening these into the parent keeps "not set" and the variants on a single
// selector rather than nesting one selector inside another.
const unionBranches = (
  schema: JSONSchema7Definition,
  definitions?: Record<string, JSONSchema7Definition>,
): JSONSchema7Definition[] | undefined => {
  if (!isJSONSchema(schema)) return undefined;

  const resolved = schema.$ref
    ? definitions?.[schema.$ref.split("/").pop() as string]
    : schema;
  if (!resolved || !isJSONSchema(resolved)) return undefined;

  const branches = resolved.oneOf ?? resolved.anyOf;
  return branches && branches.length > 0 ? branches : undefined;
};

// Function to remove `anyOf` where `null` is present
const simplifyAnyOf = (
  schema: JSONSchema7Definition,
  definitions?: Record<string, JSONSchema7Definition>,
): JSONSchema7Definition => {
  if (!isJSONSchema(schema)) return schema;

  let newSchema: JSONSchema7 = { ...schema };

  if (newSchema.anyOf) {
    const hasNullBranch = newSchema.anyOf.some(
      (s) => isJSONSchema(s) && s.type === "null",
    );
    // Remove `null` from `anyOf`
    const filteredSchemas = newSchema.anyOf.filter(
      (s) => !(isJSONSchema(s) && s.type === "null"),
    );

    // Keep "not set" representable for anything RJSF renders as a choice. A union
    // gets its branches flattened up so that "not set" and the variants share one
    // selector; anything else keeps the plain `anyOf: [X, null]` pair.
    const nullableBranching =
      hasNullBranch &&
      filteredSchemas.length === 1 &&
      isBranchingSchema(filteredSchemas[0]);

    if (nullableBranching) {
      const branches = unionBranches(filteredSchemas[0], definitions);
      const { anyOf: _anyOf, ...rest } = newSchema;
      newSchema = {
        ...rest,
        oneOf: [
          ...(branches ?? [filteredSchemas[0]]).map((branch) =>
            simplifyAnyOf(branch, definitions),
          ),
          NULL_BRANCH,
        ],
      };
    } else if (filteredSchemas.length === 1) {
      // If only one type remains, replace `anyOf` with that schema
      if (isJSONSchema(filteredSchemas[0])) {
        const originalTitle = newSchema.title;
        newSchema = { ...filteredSchemas[0] };
        // Preserve the title if missing from the new schema
        if (!newSchema.title && originalTitle) newSchema.title = originalTitle;
      }
    } else {
      newSchema.anyOf = filteredSchemas;
    }
  }

  // Definitions found here take over for nested lookups: a $ref is always
  // resolved against the root's definitions.
  const defs = newSchema.definitions ?? definitions;

  if (newSchema.properties) {
    newSchema.properties = Object.fromEntries(
      Object.entries(newSchema.properties).map(([key, value]) => [
        key,
        simplifyAnyOf(value, defs),
      ]),
    );
  }

  if (newSchema.definitions) {
    newSchema.definitions = Object.fromEntries(
      Object.entries(newSchema.definitions).map(([key, value]) => [
        key,
        simplifyAnyOf(value, defs),
      ]),
    );
  }

  if (newSchema.items) {
    if (Array.isArray(newSchema.items)) {
      newSchema.items = newSchema.items.map((item) =>
        simplifyAnyOf(item, defs),
      );
    } else {
      newSchema.items = simplifyAnyOf(newSchema.items, defs);
    }
  }

  return newSchema;
};

const consolidateOneOfToEnum = (
  schema: JSONSchema7Definition,
): JSONSchema7Definition => {
  if (!isJSONSchema(schema)) return schema;
  const newSchema: JSONSchema7 = { ...schema };

  const extractOneOfValues = (
    arr: JSONSchema7[],
  ): { values: any[]; titles: (string | undefined)[] } | null => {
    const values: any[] = [];
    const titles: (string | undefined)[] = [];
    for (const sub of arr) {
      let v: any | undefined;
      if ("const" in sub && typeof sub.const !== "undefined") v = sub.const;
      else if (Array.isArray(sub.enum) && sub.enum.length === 1)
        v = sub.enum[0];
      else return null;
      values.push(v);
      titles.push(sub.title);
    }
    return { values, titles };
  };

  if (newSchema.oneOf && newSchema.oneOf.every(isJSONSchema)) {
    const oneOfValues = extractOneOfValues(newSchema.oneOf as JSONSchema7[]);
    if (oneOfValues) {
      // Ensure the parent looks like a string/number/etc. based on first value
      if (typeof oneOfValues.values[0] === "string") {
        newSchema.type = "string";
      } else if (typeof oneOfValues.values[0] === "number") {
        newSchema.type = "number";
      }

      const hasTitles = oneOfValues.titles.some((t) => t !== undefined);
      if (hasTitles) {
        // Normalize to oneOf with const+title format so RJSF renders labeled select options
        newSchema.oneOf = oneOfValues.values.map((val, i) => ({
          const: val,
          title: oneOfValues.titles[i] || String(val),
        }));
      } else {
        // No titles available, fall back to plain enum
        delete newSchema.oneOf;
        (newSchema as JSONSchema7 & { enum: any[] }).enum = oneOfValues.values;
      }
    }
  }

  // Recursively handle nested schemas
  if (newSchema.properties) {
    newSchema.properties = Object.fromEntries(
      Object.entries(newSchema.properties).map(([key, value]) => [
        key,
        consolidateOneOfToEnum(value),
      ]),
    );
  }

  if (newSchema.items) {
    if (Array.isArray(newSchema.items)) {
      newSchema.items = newSchema.items.map(consolidateOneOfToEnum);
    } else {
      newSchema.items = consolidateOneOfToEnum(newSchema.items);
    }
  }

  if (newSchema.definitions) {
    newSchema.definitions = Object.fromEntries(
      Object.entries(newSchema.definitions).map(([k, v]) => [
        k,
        consolidateOneOfToEnum(v),
      ]),
    );
  }

  return newSchema;
};

// schemars encodes the tag of an internally tagged Rust enum as a single-member
// `enum` — an `Authentication::Basic` variant carries `type: { enum: ["basic"] }`.
// RJSF reads that as a required select with exactly one option and leaves it
// empty, so picking a variant in the union selector never fills the tag in and
// the form stays invalid on a field the user has nothing to decide about. Giving
// the tag its own value as a `default` is what RJSF populates from when the user
// picks a branch. (`const` states the same constraint but does not survive that
// path: the branch's keys arrive pre-seeded as undefined, which defeats it.)
//
// Only tags inside a union branch get this. A default anywhere else makes RJSF
// materialize the object holding it, which would conjure optional parameters — a
// `Code` field's `type`, say — into existence unasked.
// A schema's place in a union, which is what decides whether a single-member
// `enum` is a discriminator. A discriminator is always an immediate property of
// its variant, so the role advances exactly one level and then stops — letting it
// propagate deeper would treat the `type` inside a variant's nested `Code` field
// as a tag and materialize that optional field.
type UnionRole = "none" | "branch" | "branchProperty";

const defaultUnionTags = (
  schema: JSONSchema7Definition,
  role: UnionRole = "none",
): JSONSchema7Definition => {
  if (!isJSONSchema(schema)) return schema;

  const newSchema: JSONSchema7 = { ...schema };

  if (
    role === "branchProperty" &&
    Array.isArray(newSchema.enum) &&
    newSchema.enum.length === 1 &&
    typeof newSchema.default === "undefined"
  ) {
    newSchema.default = newSchema.enum[0] as JSONSchema7["default"];
  }

  // Recursively handle nested schemas
  if (newSchema.properties) {
    const childRole: UnionRole = role === "branch" ? "branchProperty" : "none";
    newSchema.properties = Object.fromEntries(
      Object.entries(newSchema.properties).map(([key, value]) => [
        key,
        defaultUnionTags(value, childRole),
      ]),
    );
  }

  if (newSchema.definitions) {
    newSchema.definitions = Object.fromEntries(
      Object.entries(newSchema.definitions).map(([key, value]) => [
        key,
        defaultUnionTags(value),
      ]),
    );
  }

  if (newSchema.items) {
    if (Array.isArray(newSchema.items)) {
      newSchema.items = newSchema.items.map((item) => defaultUnionTags(item));
    } else {
      newSchema.items = defaultUnionTags(newSchema.items);
    }
  }

  for (const keyword of ["allOf", "oneOf", "anyOf"] as const) {
    const branches = newSchema[keyword];
    if (branches) {
      // An object branch of a union is where a tag lives; a branch that is a bare
      // constant is an option of a plain enum select and owns no tag.
      newSchema[keyword] = branches.map((branch) =>
        defaultUnionTags(
          branch,
          keyword !== "allOf" && isJSONSchema(branch) && !!branch.properties
            ? "branch"
            : "none",
        ),
      );
    }
  }

  return newSchema;
};

// Nested `anyOf` inside `oneOf` needs to be simplified as `oneOf` will override `anyOf`
const simplifyAnyOfInsideOneOf = (
  schema: JSONSchema7Definition,
  definitions?: Record<string, JSONSchema7Definition>,
): JSONSchema7Definition => {
  if (!isJSONSchema(schema)) return schema;

  const newSchema: JSONSchema7 = { ...schema };

  if (newSchema.oneOf) {
    newSchema.oneOf = newSchema.oneOf.map((subSchema) => {
      if (isJSONSchema(subSchema) && subSchema.properties) {
        const updatedProperties = Object.fromEntries(
          Object.entries(subSchema.properties).map(([key, value]) => [
            key,
            simplifyAnyOf(value, definitions),
          ]),
        );
        return { ...subSchema, properties: updatedProperties };
      }
      return subSchema;
    });
  }

  return newSchema;
};

// Function to simplify `allOf` with single `$ref` - common pattern from schemars with default values
const simplifyAllOf = (
  schema: JSONSchema7Definition,
  definitions?: Record<string, JSONSchema7Definition>,
): JSONSchema7Definition => {
  if (!isJSONSchema(schema)) return schema;

  let newSchema: JSONSchema7 = { ...schema };

  // Handle allOf with single $ref (common pattern from Rust schemars with defaults)
  if (newSchema.allOf && newSchema.allOf.length === 1) {
    const subSchema = newSchema.allOf[0];
    if (isJSONSchema(subSchema) && subSchema.$ref) {
      // Extract the reference key from "#/definitions/EnumName"
      const refKey = subSchema.$ref.split("/").pop();
      if (refKey && definitions?.[refKey]) {
        const resolvedSchema = definitions[refKey];
        if (isJSONSchema(resolvedSchema)) {
          // Merge the referenced schema with the current schema, preserving properties like 'default'
          const { allOf, ...schemaWithoutAllOf } = newSchema;
          newSchema = { ...resolvedSchema, ...schemaWithoutAllOf };
        }
      }
    }
  }

  // Recursively handle nested schemas
  if (newSchema.properties) {
    newSchema.properties = Object.fromEntries(
      Object.entries(newSchema.properties).map(([key, value]) => [
        key,
        simplifyAllOf(value, definitions),
      ]),
    );
  }

  if (newSchema.items) {
    if (Array.isArray(newSchema.items)) {
      newSchema.items = newSchema.items.map((item) =>
        simplifyAllOf(item, definitions),
      );
    } else {
      newSchema.items = simplifyAllOf(newSchema.items, definitions);
    }
  }

  return newSchema;
};

export const applySchemaDefaults = <T = any>(
  schema: RJSFSchema | JSONSchema7Definition,
  formData: T,
): T => {
  const patchedSchema = patchAnyOfAndOneOfType(schema as JSONSchema7Definition);
  const schemaUtils = createSchemaUtils(validator, patchedSchema);
  return schemaUtils.getDefaultFormState(patchedSchema, formData) as T;
};

export const patchAnyOfAndOneOfType = (
  schema: JSONSchema7Definition,
): RJSFSchema => {
  if (!isJSONSchema(schema)) {
    return { type: "boolean", default: schema };
  }

  let newSchema: JSONSchema7 = { ...schema };

  // Remove `anyOf` where `null` is present
  newSchema = simplifyAnyOf(newSchema) as JSONSchema7;
  // Ensure `oneOf` does not interfere with `anyOf` simplification
  newSchema = simplifyAnyOfInsideOneOf(
    newSchema,
    newSchema.definitions,
  ) as JSONSchema7;
  // Simplify `allOf` with single `$ref` (handles Rust schemars enum defaults)
  newSchema = simplifyAllOf(newSchema, newSchema.definitions) as JSONSchema7;

  // Apply consolidateOneOfToEnum to the root schema and all nested properties
  newSchema = consolidateOneOfToEnum(newSchema) as JSONSchema7;

  // Runs last so it only sees enums consolidateOneOfToEnum left alone: a lone
  // remaining value is a fixed tag, not a choice.
  newSchema = defaultUnionTags(newSchema) as JSONSchema7;

  return newSchema;
};
