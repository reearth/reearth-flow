/**
 * Validation, against the schema the engine published.
 *
 * Nothing here sees a rewritten schema. The form's verdict and the engine's
 * verdict are therefore the same verdict — which the previous pipeline could
 * not promise, since it ran AJV over a copy with nullability stripped, `oneOf`
 * rewritten to `enum` and `allOf` merged away, and so reported valid configs
 * invalid (and the reverse).
 */
import Ajv, { type ErrorObject, type ValidateFunction } from "ajv";
import addFormats from "ajv-formats";
import type { JSONSchema7Definition } from "json-schema";

/**
 * Failures keyed by dot path; `""` holds failures on the form as a whole.
 *
 * A key with an empty list is a field that failed with nothing worth saying —
 * a required field left blank is already marked by its asterisk and its red
 * border, and a sentence underneath repeats what the form has shown twice. The
 * key's presence is what makes the field invalid; the list is only what to
 * write beneath it.
 */
export type ValidationErrors = Record<string, string[]>;

const ajv = new Ajv({
  allErrors: true,
  // The engine's schemas carry formats that describe a value rather than
  // constrain it (`code`, `uint64`, `double`); strict mode would reject them.
  strict: false,
  // A union reports one failure for the union plus one per branch it did not
  // match. Only the union's own message is useful to a reader.
  verbose: false,
  // The engine's custom formats are descriptive, so AJV's "unknown format"
  // notices say nothing a reader can act on.
  logger: false,
});
addFormats(ajv);

const compiled = new WeakMap<object, ValidateFunction>();

const validatorFor = (schema: JSONSchema7Definition): ValidateFunction => {
  if (typeof schema === "boolean") return ajv.compile({});
  const cached = compiled.get(schema);
  if (cached) return cached;
  const validate = ajv.compile(schema);
  compiled.set(schema, validate);
  return validate;
};

/** `/response/responseEncoding` → `response.responseEncoding`. */
const instancePathToKey = (instancePath: string): string =>
  instancePath === ""
    ? ""
    : instancePath
        .slice(1)
        .split("/")
        .map((segment) => segment.replace(/~1/g, "/").replace(/~0/g, "~"))
        .join(".");

/**
 * AJV reports a missing property against the parent object. Moving it onto the
 * property itself is what puts the message under the field the user has to fill.
 */
const keyForError = (error: ErrorObject): string => {
  const base = instancePathToKey(error.instancePath);
  if (error.keyword === "required") {
    const missing = (error.params as { missingProperty?: string })
      .missingProperty;
    if (missing) return base ? `${base}.${missing}` : missing;
  }
  return base;
};

/**
 * `must be null` only ever comes from the null branch of a nullable schema, so
 * it fires whenever an optional section is present but incomplete. The useful
 * complaint in that case is the child's, never this one.
 */
const isNullBranchNoise = (error: ErrorObject): boolean =>
  error.keyword === "type" &&
  (error.params as { type?: string | string[] }).type === "null";

const CHOICE_MESSAGE = "Choose one of the available options";

/** The message to write under the field, or null to show the highlight alone. */
const messageFor = (error: ErrorObject): string | null => {
  switch (error.keyword) {
    case "required":
      return null;
    case "oneOf":
    case "anyOf":
      return "Choose one of the available options";
    case "const":
    case "enum":
      return "Not one of the allowed values";
    default:
      return error.message ?? "Invalid value";
  }
};

export const validate = (
  schema: JSONSchema7Definition | undefined,
  value: unknown,
): ValidationErrors => {
  if (!schema) return {};

  let ok: boolean;
  let validator: ValidateFunction;
  try {
    validator = validatorFor(schema);
    ok = validator(value) as boolean;
  } catch (error) {
    // A schema AJV cannot compile is a problem with the schema, not the data;
    // surfacing it on the form beats failing the render.
    return { "": [`Schema could not be checked: ${(error as Error).message}`] };
  }

  if (ok || !validator.errors) return {};

  const errors: ValidationErrors = {};
  for (const error of validator.errors) {
    if (isNullBranchNoise(error)) continue;
    const key = keyForError(error);
    const existing = (errors[key] ??= []);
    const message = messageFor(error);
    if (message !== null && !existing.includes(message)) existing.push(message);
  }

  // A union that failed reports itself as well as each branch it did not match.
  // Where something more specific is already known, "choose an option" adds
  // nothing but a second red line above the field the user has to fix.
  const keys = Object.keys(errors);
  const pruned: ValidationErrors = {};
  for (const key of keys) {
    const messages = errors[key];
    const hasDetail =
      messages.includes(CHOICE_MESSAGE) &&
      keys.some((other) => other !== key && other.startsWith(`${key}.`));
    // The key stays either way: the field is still invalid, it just has
    // nothing left to say that the field below it is not saying better.
    pruned[key] = hasDetail
      ? messages.filter((message) => message !== CHOICE_MESSAGE)
      : messages;
  }

  return pruned;
};

export const isValid = (errors: ValidationErrors): boolean =>
  Object.keys(errors).length === 0;
