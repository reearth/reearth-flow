import { useCallback } from "react";

import { useProject } from "@flow/lib/gql";
import { useJob } from "@flow/lib/gql/job";
import { loadStateFromIndexedDB, useCurrentProject } from "@flow/stores";
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
  }, [currentProject, rawWorkflows, runProject]);

  const handleDebugRunStop = useCallback(async () => {
    const debugRunState = await loadStateFromIndexedDB("debugRun");
    if (!debugRunState?.jobId) return;

    console.log("stop debug run", debugRunState.jobId);
    const data = await useJobCancel(debugRunState.jobId);
    console.log("stop debug run data", data);
  }, [useJobCancel]);

  return {
    handleDebugRunStart,
    handleDebugRunStop,
  };
};
