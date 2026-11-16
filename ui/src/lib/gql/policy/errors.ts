import { ClientError } from "graphql-request";

export type PlanLimitKind =
  | "jobLimitReached"
  | "debugRunsPerHourLimit"
  | "deploymentRunsPerDayLimit"
  | "hourlyScheduleNotAllowed"
  | "enabledTriggerLimit"
  | "assetTooLarge";

export type PlanLimit = {
  kind: PlanLimitKind;
  /** The plan's limit as the server words it, such as "5MB", when it sends one. */
  limit?: string;
};

// The API reports plan limits as message text only, like the project errors in
// ../project/errors.ts. Matching lives here alone so that moving to error codes,
// once the API sends them, changes nothing else. Messages are matched as a
// substring: the server appends details, such as the IDs of the jobs holding
// the workspace's debug run slots or the asset size limit.
const KINDS_BY_MESSAGE: [string, PlanLimitKind][] = [
  [
    "this workspace already has as many jobs running as its plan allows",
    "jobLimitReached",
  ],
  [
    "this workspace's plan does not allow more debug runs this hour",
    "debugRunsPerHourLimit",
  ],
  [
    "this workspace's plan does not allow more deployment runs today",
    "deploymentRunsPerDayLimit",
  ],
  [
    "this workspace's plan does not allow hourly schedules",
    "hourlyScheduleNotAllowed",
  ],
  [
    "this workspace already has as many triggers enabled as its plan allows",
    "enabledTriggerLimit",
  ],
  ["this asset is larger than the workspace's plan allows", "assetTooLarge"],
];

// The asset size refusal ends with the plan's limit: "...: the limit is 5MB".
const LIMIT_SUFFIX = /: the limit is (.+)$/;

export const toPlanLimit = (err: unknown): PlanLimit | undefined => {
  const messages =
    err instanceof ClientError
      ? (err.response.errors ?? []).map((e) => e.message)
      : [];
  for (const message of messages) {
    const match = KINDS_BY_MESSAGE.find(([text]) => message.includes(text));
    if (match) {
      const limit = LIMIT_SUFFIX.exec(message)?.[1]?.trim();
      return limit ? { kind: match[1], limit } : { kind: match[1] };
    }
  }
  return undefined;
};

export const toPlanLimitKind = (err: unknown): PlanLimitKind | undefined =>
  toPlanLimit(err)?.kind;
