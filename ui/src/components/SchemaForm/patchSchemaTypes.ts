import { RJSFSchema } from "@rjsf/utils";
import { JSONSchema7, JSONSchema7Definition } from "json-schema";

// This is a workaround for the `anyOf` type for RJSF/JSON Schema. Currently if "null" only is passed as a type in `anyof` it won't work as expected.
// We should regualry check this issue and update RJSF once a fix is published. (https://github.com/rjsf-team/react-jsonschema-form/issues/4380)
// Update March 2025: a new function was added to check if anyOf is nested in a oneOf as it will override the anyOf simplification

// Utility function to check if schema is a valid JSONSchema
const isJSONSchema = (schema: JSONSchema7Definition): schema is JSONSchema7 =>
  typeof schema !== "boolean";

// Function to remove `anyOf` where `null` is present
const simplifyAnyOf = (
  schema: JSONSchema7Definition,
): JSONSchema7Definition => {
  if (!isJSONSchema(schema)) return schema;

  let newSchema: JSONSchema7 = { ...schema };

  if (newSchema.anyOf) {
    // Remove `null` from `anyOf`
    const filteredSchemas = newSchema.anyOf.filter(
      (s) => !(isJSONSchema(s) && s.type === "null"),
    );

    // If only one type remains, replace `anyOf` with that schema
    if (filteredSchemas.length === 1) {
      if (isJSONSchema(filteredSchemas[0])) {
        newSchema = { ...filteredSchemas[0] };
      }
    } else {
      newSchema.anyOf = filteredSchemas;
    }
  }

  if (newSchema.properties) {
    newSchema.properties = Object.fromEntries(
      Object.entries(newSchema.properties).map(([key, value]) => [
        key,
        simplifyAnyOf(value),
      ]),
    );
  }

  if (newSchema.definitions) {
    newSchema.definitions = Object.fromEntries(
      Object.entries(newSchema.definitions).map(([key, value]) => [
        key,
        simplifyAnyOf(value),
      ]),
    );
  }

  if (newSchema.items) {
    if (Array.isArray(newSchema.items)) {
      newSchema.items = newSchema.items.map(simplifyAnyOf);
    } else {
      newSchema.items = simplifyAnyOf(newSchema.items);
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
      // Force enum to ensure RJSF uses a select dropdown
      (newSchema as JSONSchema7 & { enum: any[] }).enum = oneOfValues.values;
    }
  }

  return newSchema;
};

// Nested `anyOf` inside `oneOf` needs to be simplified as `oneOf` will override `anyOf`
const simplifyAnyOfInsideOneOf = (
  schema: JSONSchema7Definition,
): JSONSchema7Definition => {
  if (!isJSONSchema(schema)) return schema;

  const newSchema: JSONSchema7 = { ...schema };

  if (newSchema.oneOf) {
    newSchema.oneOf = newSchema.oneOf.map((subSchema) => {
      if (isJSONSchema(subSchema) && subSchema.properties) {
        const updatedProperties = Object.fromEntries(
          Object.entries(subSchema.properties).map(([key, value]) => [
            key,
            simplifyAnyOf(value),
          ]),
        );
        return { ...subSchema, properties: updatedProperties };
      }
      return subSchema;
    });
  }

  return newSchema;
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
  newSchema = simplifyAnyOfInsideOneOf(newSchema) as JSONSchema7;

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
