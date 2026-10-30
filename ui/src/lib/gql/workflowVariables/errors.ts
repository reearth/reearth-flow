import { ClientError } from "graphql-request";

// The API refuses to save a scheduled trigger, or to run a deployment, when a
// required workflow variable would be left empty, and names them after this
// text. It sends message text only, as for plan limits in ../policy/errors.ts.
const MISSING_REQUIRED = "required workflow variables have no value";

// The names of the required variables an API error says have no value, or
// undefined for any other error.
export const toMissingRequiredVariables = (
  err: unknown,
): string[] | undefined => {
  if (!(err instanceof ClientError)) return undefined;
  for (const { message } of err.response.errors ?? []) {
    const at = message.indexOf(MISSING_REQUIRED);
    if (at === -1) continue;
    const names = message
      .slice(at + MISSING_REQUIRED.length)
      .replace(/^:\s*/, "")
      .split(",")
      .map((n) => n.trim())
      .filter(Boolean);
    return names;
  }
  return undefined;
};
