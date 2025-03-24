import { useEffect, useMemo } from "react";

import { useJob } from "@flow/lib/gql/job";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

import useNodeStatusSubscription from "../useNodeStatusSubscription";

export default ({ id }: { id: string }) => {
  const [currentProject] = useCurrentProject();

  const { value: debugRunState, updateValue } = useIndexedDB("debugRun");

  const { useGetJob } = useJob();

  const debugJobState = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id),
    [debugRunState, currentProject],
  );
  const { job: debugRun } = useGetJob(debugJobState?.jobId);

  const { useGetNodeExecution } = useJob();

  const { nodeExecution, refetch } = useGetNodeExecution(
    debugJobState?.jobId,
    id,
  );

  // const intermediateDataUrl = useMemo(
  //   () =>
  //     nodeExecution?.intermediateDataUrl ||
  //     debugJobState?.nodeExecutions?.find((ne) => ne.nodeId === id)
  //       ?.intermediateDataUrl,
  //   [debugJobState?.nodeExecutions, nodeExecution?.intermediateDataUrl, id],
  // );

  const { realTimeNodeStatus } = useNodeStatusSubscription({
    id,
    debugJobState,
    debugRun,
  });

  const nodeStatus = useMemo(() => {
    if (debugJobState?.nodeExecutions) {
      const node = debugJobState.nodeExecutions.find(
        (nodeExecution) => nodeExecution.nodeId === id,
      );

      if (node) {
        return node?.status;
      }
    }
    return realTimeNodeStatus;
  }, [debugJobState, realTimeNodeStatus, id]);

  useEffect(() => {
    if (
      (nodeStatus === "completed" || nodeStatus === "failed") &&
      (!nodeExecution || nodeExecution?.status !== nodeStatus)
    ) {
      (async () => {
        await refetch();
      })();
    }
  }, [nodeStatus, nodeExecution, refetch]);

  useEffect(() => {
    if (
      nodeExecution &&
      debugRunState &&
      !debugJobState?.nodeExecutions?.find(
        (ne) =>
          ne.id === nodeExecution.id && nodeExecution.status === ne.status,
      )
    ) {
      (async () =>
        await updateValue((prevState) => {
          const alreadyExists = prevState.jobs.some((job) =>
            job.nodeExecutions?.some((ne) => ne.id === nodeExecution.id),
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
                    nodeExecutions: [
                      ...(job.nodeExecutions ?? []),
                      nodeExecution,
                    ],
                  }
                : job,
            ),
          };
        }))();
    }
  }, [
    nodeExecution,
    debugJobState,
    debugRunState,
    currentProject,
    updateValue,
  ]);

  // const handleIntermediateDataSet = useCallback(async () => {
  //   if (!intermediateDataUrl) return;
  //   const newDebugRunState: DebugRunState = {
  //     ...debugRunState,
  //     jobs:
  //       debugRunState?.jobs?.map((job) =>
  //         job.projectId === currentProject?.id
  //           ? {
  //               ...job,
  //               selectedIntermediateData: {
  //                 edgeId: id,
  //                 url: intermediateDataUrl,
  //               },
  //             }
  //           : job,
  //       ) ?? [],
  //   };
  //   await updateValue(newDebugRunState);
  // }, [intermediateDataUrl, debugRunState, currentProject, id, updateValue]);

  // const nodeExecution: NodeExecution | undefined = {
  //   id: "execution1",
  //   jobId: "job1",
  //   nodeId: "node-1",
  //   status: "running",
  //   startedAt: "2021-08-02T00:00:00Z",
  //   // completedAt: "2021-08-02T00:00:00Z",
  //   intermediateDataUrl: "https://example.com",
  // };

  return {
    nodeExecution,
    // handleIntermediateDataSet,
  };
};
