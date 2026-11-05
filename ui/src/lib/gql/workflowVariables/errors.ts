import { ClientError } from "graphql-request";

// The API refuses to save a scheduled trigger, or to run a deployment, when a
// required workflow variable would be left empty, and names them after this
// text. It sends message text only, as for plan limits in ../policy/errors.ts.
const MISSING_REQUIRED = "required workflow variables have no value";
// A deployment run given a variable the deployment does not define.
const UNKNOWN = "workflow variables not defined by the deployment";
// A deployment run given a value that does not fit its variable's type.
const INVALID_VALUE = "workflow variable value does not fit its type";

// The names an API error lists after `text`, or undefined if no error says it.
// Several refusals can arrive joined in one message, one per line, so the list
// ends at the line's end.
const namesAfter = (err: unknown, text: string): string[] | undefined => {
  if (!(err instanceof ClientError)) return undefined;
  for (const { message } of err.response.errors ?? []) {
    const at = message.indexOf(text);
    if (at === -1) continue;
    return message
      .slice(at + text.length)
      .split("\n")[0]
      .replace(/^:\s*/, "")
      .split(",")
      .map((n) => n.trim())
      .filter(Boolean);
  }
  return undefined;
};

// The names of the required variables an API error says have no value, or
// undefined for any other error.
export const toMissingRequiredVariables = (
  err: unknown,
): string[] | undefined => namesAfter(err, MISSING_REQUIRED);

// The variables an API error says the deployment does not define.
export const toUnknownVariables = (err: unknown): string[] | undefined =>
  namesAfter(err, UNKNOWN);

// The variables an API error says were given a value of the wrong type.
export const toInvalidVariableValues = (err: unknown): string[] | undefined =>
  namesAfter(err, INVALID_VALUE);
