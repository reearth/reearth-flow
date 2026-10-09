import { ClientError } from "graphql-request";

import type { IntermediateDataViewErrorKind } from "@flow/types";

/**
 * A view that could not be requested. A view that rendered and has nothing to
 * show is not an error: it arrives as a view with a non-ready status.
 */
export class IntermediateDataViewError extends Error {
  readonly kind: IntermediateDataViewErrorKind;

  constructor(kind: IntermediateDataViewErrorKind, message?: string) {
    super(message ?? kind);
    this.name = "IntermediateDataViewError";
    this.kind = kind;
  }
}

// The API reports these as message text only, each starting with the fixed
// text of the server error it wraps. Matching lives here alone so that moving
// to error codes, once the API sends them, changes nothing else.
const KINDS_BY_MESSAGE_PREFIX: [string, IntermediateDataViewErrorKind][] = [
  ["the view did not finish rendering in time", "timedOut"],
  ["intermediate data views are not available", "unavailable"],
  ["job has not finished", "jobNotFinished"],
  ["intermediate data not found", "dataNotFound"],
  ["operation denied", "permissionDenied"],
];

export const toIntermediateDataViewError = (
  err: unknown,
): IntermediateDataViewError => {
  if (err instanceof IntermediateDataViewError) return err;

  const messages =
    err instanceof ClientError
      ? (err.response.errors ?? []).map((e) => e.message)
      : [];
  for (const message of messages) {
    const match = KINDS_BY_MESSAGE_PREFIX.find(([prefix]) =>
      message.startsWith(prefix),
    );
    if (match) return new IntermediateDataViewError(match[1], message);
  }

  const fallback =
    messages[0] ?? (err instanceof Error ? err.message : String(err));
  return new IntermediateDataViewError("unknown", fallback);
};
