import { useCallback } from "react";

import { useProject } from "@flow/lib/gql";
import { useJob } from "@flow/lib/gql/job";
import {
  loadStateFromIndexedDB,
  updateJobs,
  useCurrentProject,
} from "@flow/stores";
import type { Workflow } from "@flow/types";
import { createEngineReadyWorkflow } from "@flow/utils/toEngineWorkflow/engineReadyWorkflow";

export default ({ rawWorkflows }: { rawWorkflows: Workflow[] }) => {
  const [currentProject] = useCurrentProject();

  const { runProject } = useProject();
  const { useJobCancel } = useJob();

  const handleDebugRunStart = useCallback(async () => {
    console.log("start debug run");
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

    console.log("job started: ", data.job);
    if (data.job) {
      await updateJobs({ projectId: currentProject.id, jobId: data.job.id });
      // TODO: open logs panel
    }
  }, [currentProject, rawWorkflows, runProject]);

  const handleDebugRunStop = useCallback(async () => {
    const debugRunState = await loadStateFromIndexedDB("debugRun");
    const debugJob = debugRunState?.jobs?.find(
      (job) => job.projectId === currentProject?.id,
    );
    if (!debugJob) return;

    console.log("stop debug run", debugJob);
    const data = await useJobCancel(debugJob.jobId);
    if (data.isSuccess && currentProject?.id) {
      await updateJobs({ projectId: currentProject.id });
    }
  }, [currentProject?.id, useJobCancel]);

  return {
    handleDebugRunStart,
    handleDebugRunStop,
  };
};
