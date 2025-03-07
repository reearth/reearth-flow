import { useReactFlow } from "@xyflow/react";
import { useCallback } from "react";

import { useProject } from "@flow/lib/gql";
import { useJob } from "@flow/lib/gql/job";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { JobState, useCurrentProject } from "@flow/stores";
import type { Workflow } from "@flow/types";
import { createEngineReadyWorkflow } from "@flow/utils/toEngineWorkflow/engineReadyWorkflow";

export default ({ rawWorkflows }: { rawWorkflows: Workflow[] }) => {
  const [currentProject] = useCurrentProject();

  const { fitView } = useReactFlow();

  const { runProject } = useProject();
  const { useJobCancel } = useJob();

  const { value: debugRunState, updateValue } = useIndexedDB("debugRun");

  const handleDebugRunStart = useCallback(async () => {
    if (!currentProject) return;

    const engineReadyWorkflow = createEngineReadyWorkflow(
      currentProject.name,
      rawWorkflows,
    );

    if (!engineReadyWorkflow) return;

    const data = await runProject(
      currentProject.id,
      currentProject.workspaceId,
      engineReadyWorkflow,
    );

    if (data.job) {
      let jobs: JobState[] = debugRunState?.jobs || [];

      if (!data.job.id) {
        jobs =
          debugRunState?.jobs?.filter(
            (job) => job.projectId !== currentProject.id,
          ) || [];
      } else if (
        debugRunState?.jobs?.some((job) => job.projectId === currentProject.id)
      ) {
        jobs = debugRunState.jobs.map((job) => {
          if (job.projectId === currentProject.id && data.job) {
            return { projectId: currentProject.id, jobId: data.job.id };
          }
          return job;
        });
      } else {
        jobs.push({ projectId: currentProject.id, jobId: data.job.id });
      }
      await updateValue({ jobs });

      fitView({ duration: 400, padding: 0.5 });
    }
  }, [
    currentProject,
    rawWorkflows,
    debugRunState?.jobs,
    fitView,
    updateValue,
    runProject,
  ]);

  const handleDebugRunStop = useCallback(async () => {
    const debugJob = debugRunState?.jobs?.find(
      (job) => job.projectId === currentProject?.id,
    );
    if (!debugJob) return;

    console.log("stop debug run", debugJob);
    const data = await useJobCancel(debugJob.jobId);
    if (data.isSuccess && currentProject?.id) {
      const jobs: JobState[] =
        debugRunState?.jobs?.filter((j) => j.projectId !== currentProject.id) ||
        [];
      updateValue({ jobs });
    }
  }, [currentProject?.id, debugRunState?.jobs, updateValue, useJobCancel]);

  return {
    handleDebugRunStart,
    handleDebugRunStop,
  };
};
