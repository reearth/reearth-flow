import { useMemo } from "react";

import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

export default () => {
  const [currentProject] = useCurrentProject();

  const { value: debugRunState } = useIndexedDB("debugRun");

  const debugJobState = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id),
    [debugRunState, currentProject],
  );

  // COMMENTED OUT CODE: can be used with nodeExecutions and realTimeNodeStatus.
  // Currently backend isn't ready so this code is commented out.Remove code connected
  // to jobStatus when backend is ready.

  // const { useGetJob } = useJob();

  // const { job: debugRun } = useGetJob(debugJobState?.jobId);

  // const { useGetNodeExecution } = useJob();

  // const { nodeExecution, refetch } = useGetNodeExecution(
  //   debugJobState?.jobId,
  //   id,
  // );

  // const { realTimeNodeStatus } = useNodeStatusSubscription({
  //   id,
  //   debugJobState,
  //   debugRun,
  // });

  // const nodeStatus = useMemo(() => {
  //   if (debugJobState?.nodeExecutions) {
  //     const node = debugJobState.nodeExecutions.find(
  //       (nodeExecution) => nodeExecution.nodeId === id,
  //     );

  //     if (node) {
  //       return node?.status;
  //     }
  //   }
  //   return realTimeNodeStatus;
  // }, [debugJobState, realTimeNodeStatus, id]);

  // useEffect(() => {
  //   if (
  //     (nodeStatus === "completed" || nodeStatus === "failed") &&
  //     (!nodeExecution || nodeExecution?.status !== nodeStatus)
  //   ) {
  //     (async () => {
  //       await refetch();
  //     })();
  //   }
  // }, [nodeStatus, nodeExecution, refetch]);

  // useEffect(() => {
  //   if (
  //     nodeExecution &&
  //     debugRunState &&
  //     !debugJobState?.nodeExecutions?.find(
  //       (ne) =>
  //         ne.id === nodeExecution.id && nodeExecution.status === ne.status,
  //     )
  //   ) {
  //     (async () =>
  //       await updateValue((prevState) => {
  //         const alreadyExists = prevState.jobs.some((job) =>
  //           job.nodeExecutions?.some((ne) => ne.id === nodeExecution.id),
  //         );

  //         if (alreadyExists) {
  //           return prevState;
  //         }
  //         return {
  //           ...prevState,
  //           jobs: prevState.jobs.map((job) =>
  //             job.projectId === currentProject?.id
  //               ? {
  //                   ...job,
  //                   nodeExecutions: [
  //                     ...(job.nodeExecutions ?? []),
  //                     nodeExecution,
  //                   ],
  //                 }
  //               : job,
  //           ),
  //         };
  //       }))();
  //   }
  // }, [
  //   nodeExecution,
  //   debugJobState,
  //   debugRunState,
  //   currentProject,
  //   updateValue,
  // ]);
  // COMMENTED OUT CODE: can be used with nodeExecutions and realTimeNodeStatus. END

  const jobStatus = useMemo(() => debugJobState?.status, [debugJobState]);

  return {
    // nodeStatus,
    jobStatus,
  };
};
