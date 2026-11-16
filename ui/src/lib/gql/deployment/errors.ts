import { ClientError } from "graphql-request";

const HAS_TRIGGERS_CODE = "DEPLOYMENT_HAS_TRIGGERS";
const HAS_TRIGGERS_MESSAGE = "deployment has active triggers";

export const isDeploymentHasTriggersError = (err: unknown): boolean => {
  if (!(err instanceof ClientError)) return false;
  return (err.response.errors ?? []).some(
    (e) =>
      e.extensions?.code === HAS_TRIGGERS_CODE ||
      e.message.includes(HAS_TRIGGERS_MESSAGE),
  );
};
