import { useNodes } from "@xyflow/react";
import { useEffect, useMemo } from "react";

import { useJob } from "@flow/lib/gql/job";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";

export default ({ id, source }: { id: string; source: string }) => {
  const [currentProject] = useCurrentProject();

  const { value: debugRunState, updateValue } = useIndexedDB("debugRun");

  const debugJobState = useMemo(
    () =>
      debugRunState?.jobs?.find((job) => job.projectId === currentProject?.id),
    [debugRunState, currentProject],
  );

  const { useGetEdgeExecution } = useJob();

  const { edgeExecution, refetch } = useGetEdgeExecution(
    debugJobState?.jobId,
    id,
  );

  console.log("Edgeexecution", edgeExecution); // TODO: delete
  const intermediateDataUrl = useMemo(
    () =>
      edgeExecution?.intermediateDataUrl ||
      debugJobState?.edgeExecutions?.find((ee) => ee.edgeId === id)
        ?.intermediateDataUrl,
    [debugJobState?.edgeExecutions, edgeExecution?.intermediateDataUrl, id],
  );

  useEffect(() => {
    if (
      (debugJobState?.status === "completed" ||
        debugJobState?.status === "cancelled" ||
        debugJobState?.status === "failed") &&
      !edgeExecution
    ) {
      (async () => {
        const ee = await refetch();
        console.log("refetched edge execution", ee); // TODO: delete
      })();
    }
  }, [debugJobState?.status, edgeExecution, refetch]);

  useEffect(() => {
    if (
      edgeExecution &&
      debugRunState &&
      !debugJobState?.edgeExecutions?.find((ee) => ee.id === edgeExecution.id)
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

  const sourceNodeStatus = useMemo(() => {
    if (!debugJobState?.nodeExecutions) return undefined;
    const nodes = useNodes();
    const sourceNode = nodes.find((node) => node.id === source);

    console.log("sourceNode", sourceNode); // TODO: delete
    return debugJobState?.nodeExecutions?.find(
      (nodeExecution) => nodeExecution.nodeId === sourceNode?.id,
    )?.status;
  }, [debugJobState?.nodeExecutions, source]);

  return {
    sourceNodeStatus,
    intermediateDataUrl,
  };
};
