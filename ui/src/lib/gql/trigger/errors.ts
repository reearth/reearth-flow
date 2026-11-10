import { ClientError } from "graphql-request";

// The API reports this as message text only, like the plan limits in
// ../policy/errors.ts.
const SCHEDULE_NOT_UPDATED = "its schedule could not be updated";

// isScheduleNotUpdated is an update that saved the trigger, but could not
// bring its schedule in line. Saving the trigger again retries it.
export const isScheduleNotUpdated = (err: unknown): boolean =>
  err instanceof ClientError &&
  (err.response.errors ?? []).some((e) =>
    e.message.includes(SCHEDULE_NOT_UPDATED),
  );
