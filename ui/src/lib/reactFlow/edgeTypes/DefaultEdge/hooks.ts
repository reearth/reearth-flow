import { useCallback, useEffect, useMemo } from "react";

import { useJob } from "@flow/lib/gql/job";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { DebugRunState, useCurrentProject } from "@flow/stores";

import useEdgeStatusSubscription from "./useEdgeStatusSubscription";

export default ({ id, selected }: { id: string; selected?: boolean }) => {
  const [currentProject] = useCurrentProject();

  const { value: debugRunState, updateValue } = useIndexedDB("debugRun");

  const { useGetJob } = useJob();

  const debugJobState = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id),
    [debugRunState, currentProject],
  );
  const { job: debugRun } = useGetJob(debugJobState?.jobId);

  const { useGetEdgeExecution } = useJob();

  const { edgeExecution, refetch } = useGetEdgeExecution(
    debugJobState?.jobId,
    id,
  );

  const intermediateDataUrl = useMemo(
    () =>
      edgeExecution?.intermediateDataUrl ||
      debugJobState?.edgeExecutions?.find((ee) => ee.edgeId === id)
        ?.intermediateDataUrl,
    [debugJobState?.edgeExecutions, edgeExecution?.intermediateDataUrl, id],
  );

  const { realTimeEdgeStatus } = useEdgeStatusSubscription({
    id,
    debugJobState,
    debugRun,
  });

  const edgeStatus = useMemo(() => {
    if (debugJobState?.edgeExecutions) {
      const edge = debugJobState.edgeExecutions.find(
        (edgeExecution) => edgeExecution.edgeId === id,
      );

      if (edge) {
        return edge?.status;
      }
    }
    return realTimeEdgeStatus;
  }, [debugJobState, realTimeEdgeStatus, id]);

  useEffect(() => {
    if (
      (edgeStatus === "completed" || edgeStatus === "failed") &&
      (!edgeExecution || edgeExecution?.status !== edgeStatus)
    ) {
      refetch();
    }
  }, [edgeStatus, edgeExecution, refetch]);

  useEffect(() => {
    if (
      edgeExecution &&
      debugRunState &&
      !debugJobState?.edgeExecutions?.find(
        (ee) =>
          ee.id === edgeExecution.id && edgeExecution.status === ee.status,
      )
    ) {
      (async () =>
        await updateValue((prevState) => {
          const alreadyExists = prevState.jobs.some((job) =>
            job.edgeExecutions?.some((ee) => ee.id === edgeExecution.id),
          );

          if (alreadyExists) {
            return prevState;
          }
          return {
            ...prevState,
            jobs: prevState.jobs.map((job) =>
              job.projectId === currentProject?.id
                ? {
                    ...job,
                    edgeExecutions: [
                      ...(job.edgeExecutions ?? []),
                      edgeExecution,
                    ],
                  }
                : job,
            ),
          };
        }))();
    }
  }, [
    edgeExecution,
    debugJobState,
    debugRunState,
    currentProject,
    updateValue,
  ]);

  const handleIntermediateDataSet = useCallback(async () => {
    if (!selected || !intermediateDataUrl) return;
    const newDebugRunState: DebugRunState = {
      ...debugRunState,
      jobs:
        debugRunState?.jobs?.map((job) =>
          job.projectId === currentProject?.id
            ? {
                ...job,
                selectedIntermediateData: {
                  edgeId: id,
                  url: intermediateDataUrl,
                },
              }
            : job,
        ) ?? [],
    };
    await updateValue(newDebugRunState);
  }, [
    selected,
    intermediateDataUrl,
    debugRunState,
    currentProject,
    id,
    updateValue,
  ]);

  return {
    edgeStatus,
    intermediateDataUrl,
    handleIntermediateDataSet,
  };
};
