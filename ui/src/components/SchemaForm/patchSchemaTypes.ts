import { RJSFSchema } from "@rjsf/utils";
import { JSONSchema7, JSONSchema7Definition } from "json-schema";

// This is a workaround for the `anyOf` type for RJSF/JSON Schema. Currently if "null" only is passed as a type in `anyof` it won't work as expected.
// We should regualry check this issue and update RJSF once a fix is published. (https://github.com/rjsf-team/react-jsonschema-form/issues/4380)
const isJSONSchema = (schema: JSONSchema7Definition): schema is JSONSchema7 =>
  typeof schema !== "boolean";

export const patchAnyOfType = (schema: JSONSchema7Definition): RJSFSchema => {
  if (!isJSONSchema(schema)) {
    return { type: "boolean", default: schema };
  }

  const newSchema: JSONSchema7 = { ...schema };

  if (newSchema.properties) {
    newSchema.properties = Object.entries(newSchema.properties).reduce(
      (acc, [key, value]) => ({
        ...acc,
        [key]: patchAnyOfType(value),
      }),
      {},
    );
  }

  if (newSchema.definitions) {
    newSchema.definitions = Object.entries(newSchema.definitions).reduce(
      (acc, [key, value]) => ({
        ...acc,
        [key]: patchAnyOfType(value),
      }),
      {},
    );
  }

  if (newSchema.anyOf) {
    const refSchema = newSchema.anyOf.find((s) => isJSONSchema(s) && s.$ref);
    const nullSchema = newSchema.anyOf.find(
      (s) => isJSONSchema(s) && s.type === "null",
    );

    if (refSchema && nullSchema && isJSONSchema(refSchema)) {
      delete newSchema.anyOf;
      newSchema.type = ["string", "null"];
      newSchema.$ref = refSchema.$ref;
    }
  }

  if (newSchema.items) {
    if (Array.isArray(newSchema.items)) {
      newSchema.items = newSchema.items.map((item) => patchAnyOfType(item));
    } else {
      newSchema.items = patchAnyOfType(newSchema.items);
    }
  }

  return newSchema as RJSFSchema;
};
