import { useCallback, useEffect, useMemo, useState } from "react";

import { useAuth } from "@flow/lib/auth";
import { OnEdgeStatusChangeSubscription } from "@flow/lib/gql/__gen__/graphql";
import { toEdgeStatus } from "@flow/lib/gql/convert";
import { useJob } from "@flow/lib/gql/job";
import { useSubscription } from "@flow/lib/gql/subscriptions/useSubscription";
import { useSubscriptionSetup } from "@flow/lib/gql/subscriptions/useSubscriptionSetup";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { DebugRunState, useCurrentProject } from "@flow/stores";

export default ({ id, selected }: { id: string; selected?: boolean }) => {
  const [currentProject] = useCurrentProject();

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

  const { value: debugRunState, updateValue } = useIndexedDB("debugRun");

  const { useGetJob } = useJob();
  const debugJobState = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id),
    [debugRunState, currentProject],
  );
  const { job: debugRun } = useGetJob(debugJobState?.jobId);

  const intermediateDataUrl = useMemo(
    () =>
      debugRun?.status === "completed" &&
      debugRun?.edgeExecutions?.find((edge) => edge.id === id)
        ?.intermediateDataUrl,
    [debugRun, id],
  );

  const subscriptionVariables = useMemo(
    () => ({ jobId: debugJobState?.jobId, edgeId: id }),
    [debugJobState?.jobId, id],
  );

  const subscriptionDataFormatter = useCallback(
    (data: OnEdgeStatusChangeSubscription) => {
      return toEdgeStatus(data.edgeStatus);
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

  const edgeStatus = useMemo(() => {
    if (debugRun?.edgeExecutions) {
      const edge = debugRun.edgeExecutions.find(
        (edgeExecution) => edgeExecution.id === id,
      );
      return edge?.status;
    }
    return realTimeEdgeStatus;
  }, [debugRun, realTimeEdgeStatus, id]);

  const handleIntermediateDataSet = useCallback(() => {
    if (!selected) return;
    const newDebugRunState: DebugRunState = {
      ...debugRunState,
      jobs:
        debugRunState?.jobs?.map((job) =>
          job.projectId === currentProject?.id
            ? {
                ...job,
                selectedIntermediateData: {
                  edgeId: id,
                  url: "/7571eea0-eabf-4ff7-b978-e5965d882409.jsonl", //TODO: replace with actual intermediate data
                },
              }
            : job,
        ) ?? [],
    };
    updateValue(newDebugRunState);
  }, [selected, debugRunState, currentProject, id, updateValue]);

  return {
    edgeStatus,
    intermediateDataUrl,
    handleIntermediateDataSet,
  };
};
