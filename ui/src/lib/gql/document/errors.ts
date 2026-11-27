import { ClientError } from "graphql-request";

// The API names a recovery entry it can no longer rebuild exactly by this
// `extensions.code`, which stays fixed when the message is reworded.
export const VERSION_NOT_REBUILDABLE_CODE = "VERSION_NOT_REBUILDABLE";

/** Reports whether a preview or rollback was refused because the entry cannot be rebuilt. */
export const isVersionNotRebuildable = (err: unknown): boolean =>
  err instanceof ClientError &&
  (err.response.errors ?? []).some(
    (e) => e.extensions?.code === VERSION_NOT_REBUILDABLE_CODE,
  );
