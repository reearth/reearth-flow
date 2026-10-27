import { ClientError } from "graphql-request";

export type PlanLimitKind = "jobLimitReached" | "hourlyScheduleNotAllowed";

// The API reports plan limits as message text only, like the project errors in
// ../project/errors.ts. Matching lives here alone so that moving to error codes,
// once the API sends them, changes nothing else. The job message is matched as
// a prefix: the server appends the IDs of the jobs holding the workspace's
// slots.
const KINDS_BY_MESSAGE: [string, PlanLimitKind][] = [
  [
    "this workspace already has as many jobs running as its plan allows",
    "jobLimitReached",
  ],
  [
    "this workspace's plan does not allow hourly schedules",
    "hourlyScheduleNotAllowed",
  ],
];

export const toPlanLimitKind = (err: unknown): PlanLimitKind | undefined => {
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
