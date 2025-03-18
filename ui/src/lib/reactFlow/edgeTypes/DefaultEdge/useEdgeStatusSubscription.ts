import { useCallback, useEffect, useMemo, useState } from "react";

import { useAuth } from "@flow/lib/auth";
import { OnEdgeStatusChangeSubscription } from "@flow/lib/gql/__gen__/graphql";
import { useSubscription } from "@flow/lib/gql/subscriptions/useSubscription";
import { useSubscriptionSetup } from "@flow/lib/gql/subscriptions/useSubscriptionSetup";
import { JobState } from "@flow/stores";
import type { Job } from "@flow/types";

export default ({
  id,
  debugJobState,
  debugRun,
}: {
  id: string;
  debugJobState: JobState | undefined;
  debugRun: Job | undefined;
}) => {
  const { getAccessToken } = useAuth();
  const [accessToken, setAccessToken] = useState<string | undefined>(undefined);

  useEffect(() => {
    if (!accessToken) {
      (async () => {
        const token = await getAccessToken();
        setAccessToken(token);
      })();
    }
  }, [accessToken, getAccessToken]);

  const subscriptionVariables = useMemo(
    () => ({ jobId: debugJobState?.jobId, edgeId: id }),
    [debugJobState?.jobId, id],
  );

  const subscriptionDataFormatter = useCallback(
    (data: OnEdgeStatusChangeSubscription) => {
      return data.edgeStatus;
    },
    [],
  );

  useSubscriptionSetup<OnEdgeStatusChangeSubscription>(
    "GetSubscribedEdgeStatus",
    accessToken,
    subscriptionVariables,
    id,
    subscriptionDataFormatter,
    !id || !debugRun,
  );

  const { data: realTimeEdgeStatus } = useSubscription(
    "GetSubscribedEdgeStatus",
    id,
    !id || !debugRun,
  );
  return {
    realTimeEdgeStatus,
  };
};
