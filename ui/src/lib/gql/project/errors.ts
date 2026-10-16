import { ClientError } from "graphql-request";

export type ProjectErrorKind =
  | "privateNotAllowed"
  | "aliasTaken"
  | "aliasReserved"
  | "aliasInvalid"
  | "tooManyTopics"
  | "topicTooLong";

// The API reports these as message text only. Matching lives here alone so
// that moving to error codes, once the API sends them, changes nothing else.
// "aliasReserved" comes before "aliasInvalid" because both start with
// "invalid alias".
const KINDS_BY_MESSAGE: [string, ProjectErrorKind][] = [
  [
    "this workspace's plan does not allow private projects",
    "privateNotAllowed",
  ],
  ["project alias is already used by another project", "aliasTaken"],
  ["invalid alias: the 'p-' prefix is reserved", "aliasReserved"],
  ["invalid alias", "aliasInvalid"],
  ["too many topics", "tooManyTopics"],
  ["topic is too long", "topicTooLong"],
];

export const toProjectErrorKind = (
  err: unknown,
): ProjectErrorKind | undefined => {
  const messages =
    err instanceof ClientError
      ? (err.response.errors ?? []).map((e) => e.message)
      : [];
  for (const message of messages) {
    const match = KINDS_BY_MESSAGE.find(([text]) => message.includes(text));
    if (match) return match[1];
  }
  return undefined;
};
