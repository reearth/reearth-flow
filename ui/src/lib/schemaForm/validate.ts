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

import { getAtPath, pathKey } from "./path";
import type { FieldPath } from "./types";

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
  if (typeof schema === "boolean") return ajv.compile(schema);
  const cached = compiled.get(schema);
  if (cached) return cached;
  const validate = ajv.compile(schema);
  compiled.set(schema, validate);
  return validate;
};

/**
 * AJV reports a location as a JSON Pointer. Turning it into the dot path a
 * field is keyed by has to go through `pathKey`, or the two spellings diverge:
 * a map key containing a dot is escaped on the field's side and was not here,
 * so its error never reached the field it belonged to.
 *
 * Whether a segment is an array index cannot be read off the pointer, so the
 * value is walked alongside it — the container says which it is.
 */
const instancePathToPath = (instancePath: string, root: unknown): FieldPath => {
  if (instancePath === "") return [];

  const segments = instancePath
    .slice(1)
    .split("/")
    .map((segment) => segment.replace(/~1/g, "/").replace(/~0/g, "~"));

  const path: (string | number)[] = [];
  let current: unknown = root;
  for (const segment of segments) {
    if (Array.isArray(current)) {
      const index = Number(segment);
      const isIndex = Number.isInteger(index) && index >= 0;
      path.push(isIndex ? index : segment);
      current = isIndex ? current[index] : undefined;
    } else {
      path.push(segment);
      current =
        current && typeof current === "object"
          ? (current as Record<string, unknown>)[segment]
          : undefined;
    }
  }
  return path;
};

/**
 * AJV reports a missing property against the parent object. Moving it onto the
 * property itself is what puts the message under the field the user has to fill.
 */
const keyForError = (error: ErrorObject, root: unknown): string => {
  const path = instancePathToPath(error.instancePath, root);
  if (error.keyword === "required") {
    const missing = (error.params as { missingProperty?: string })
      .missingProperty;
    if (missing) return pathKey([...path, missing]);
  }
  return pathKey(path);
};

/**
 * `must be null` raised by the null *branch* of a nullable union, which fires
 * whenever an optional section is present but incomplete. The useful complaint
 * in that case is the child's, never this one.
 *
 * The branch has to be confirmed rather than assumed: a schema whose only
 * permitted value is `null` raises exactly the same error for a wrong value,
 * and dropping it there left the form with no errors at all and a verdict of
 * valid. AJV says which is which in `schemaPath` — a branch of a choice ends
 * `/anyOf/<n>/type` or `/oneOf/<n>/type`, a standalone `null` schema just
 * `/type`.
 */
const NULL_BRANCH = /\/(anyOf|oneOf)\/\d+\/type$/;

const isNullBranchNoise = (error: ErrorObject): boolean =>
  error.keyword === "type" &&
  (error.params as { type?: string | string[] }).type === "null" &&
  NULL_BRANCH.test(error.schemaPath);

const CHOICE_MESSAGE = "Choose one of the available options";

/**
 * An error raised while AJV was trying one branch of a choice, as opposed to
 * one about the value itself.
 */
const BRANCH_INTERNAL = /\/(anyOf|oneOf)\/\d+\//;

/**
 * A field holding an explicit `null` where the schema permits none.
 *
 * That is how a chosen-but-unfilled field is recorded — picking a union variant
 * marks its required keys `null` so the choice reaches other editors — so it
 * means "not filled in yet", exactly like a missing required property, and is
 * shown the same way: by the asterisk, with nothing written underneath. It is
 * still invalid, which is what keeps Update disabled.
 */
const isUnfilled = (error: ErrorObject, root: unknown): boolean =>
  error.keyword === "type" &&
  getAtPath(root, instancePathToPath(error.instancePath, root)) === null;

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

  // Where a choice failed, AJV reports it once for the choice and again for
  // every branch it tried. A branch's own complaint lands on the choice itself
  // — "must be string" from a `"euclidean"` branch, while the user is filling
  // in the CRS one — and describes a shape they did not pick. Only the errors
  // that land deeper, on a field of the branch they did pick, are about
  // anything they can act on.
  const choicePaths = new Set(
    validator.errors
      .filter((error) => error.keyword === "oneOf" || error.keyword === "anyOf")
      .map((error) => error.instancePath),
  );

  const errors: ValidationErrors = {};
  // Messages that came from trying one branch, by the path they landed on.
  // Whether they are worth showing depends on what else was found — see below.
  const branchOnly: Record<string, Set<string>> = {};

  for (const error of validator.errors) {
    if (isNullBranchNoise(error)) continue;
    const key = keyForError(error, value);
    const existing = (errors[key] ??= []);
    const message = isUnfilled(error, value) ? null : messageFor(error);
    if (message === null) continue;
    if (!existing.includes(message)) existing.push(message);

    if (
      BRANCH_INTERNAL.test(error.schemaPath) &&
      // `required` re-keys onto the missing property, which is deeper than the
      // choice and is always about the user's own value.
      error.keyword !== "required" &&
      choicePaths.has(error.instancePath)
    ) {
      (branchOnly[key] ??= new Set()).add(message);
    }
  }

  // A choice that failed reports itself, and again for every branch it tried.
  // Once something deeper is known — a field of the branch the user is actually
  // filling in — the shallow reports are all noise: "choose an option" above a
  // field that already says what is wrong, and "must be string" from a branch
  // they did not pick.
  //
  // With nothing deeper to go on, they are all there is, and a plain enum has
  // no depth at all — so "Not one of the allowed values" survives there.
  const keys = Object.keys(errors);
  const pruned: ValidationErrors = {};
  for (const key of keys) {
    const messages = errors[key];
    const hasDetail = keys.some(
      (other) => other !== key && other.startsWith(`${key}.`),
    );
    const noise = branchOnly[key];
    // The key stays either way: the field is still invalid, it just has nothing
    // left to say that the field below it is not saying better.
    pruned[key] = hasDetail
      ? messages.filter(
          (message) => message !== CHOICE_MESSAGE && !noise?.has(message),
        )
      : messages;
  }

  return pruned;
};

export const isValid = (errors: ValidationErrors): boolean =>
  Object.keys(errors).length === 0;
