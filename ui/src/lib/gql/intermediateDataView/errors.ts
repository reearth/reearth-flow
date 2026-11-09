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

// The API names these errors by `extensions.code`, which stays fixed when a
// message is reworded.
const KINDS_BY_CODE: Record<string, IntermediateDataViewErrorKind> = {
  VIEW_RENDER_TIMED_OUT: "timedOut",
  VIEWS_UNAVAILABLE: "unavailable",
  JOB_NOT_FINISHED: "jobNotFinished",
  INTERMEDIATE_DATA_NOT_FOUND: "dataNotFound",
  OPERATION_DENIED: "permissionDenied",
  VIEW_WORKERS_BUSY: "workersBusy",
};

// An API from before the codes sends message text only, each starting with
// the fixed text of the server error it wraps.
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

  const errors = err instanceof ClientError ? (err.response.errors ?? []) : [];
  for (const e of errors) {
    const code = e.extensions?.code;
    const kind = typeof code === "string" ? KINDS_BY_CODE[code] : undefined;
    if (kind) return new IntermediateDataViewError(kind, e.message);
  }

  const messages = errors.map((e) => e.message);
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
