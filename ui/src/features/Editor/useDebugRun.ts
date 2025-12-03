import { useReactFlow } from "@xyflow/react";
import { useCallback } from "react";
import type { Awareness } from "y-protocols/awareness";

import { useProject, useProjectVariables } from "@flow/lib/gql";
import { useJob } from "@flow/lib/gql/job";
import { useT } from "@flow/lib/i18n";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { JobState, useCurrentProject } from "@flow/stores";
import type { Workflow } from "@flow/types";
import { createEngineReadyWorkflow } from "@flow/utils/toEngineWorkflow/engineReadyWorkflow";

import { toast } from "../NotificationSystem/useToast";

import useDebugAwareness from "./useDebugAwareness";

export default ({
  rawWorkflows,
  yAwareness,
}: {
  rawWorkflows: Workflow[];
  yAwareness: Awareness;
}) => {
  const t = useT();
  const [currentProject] = useCurrentProject();
  const { activeDebugRuns, broadcastDebugRun } = useDebugAwareness({
    yAwareness,
    projectId: currentProject?.id,
  });
  const { useGetProjectVariables } = useProjectVariables();
  const { projectVariables } = useGetProjectVariables(currentProject?.id ?? "");

  const { fitView } = useReactFlow();

  const { runProject } = useProject();
  const { useJobCancel } = useJob();

  const { value: debugRunState, updateValue } = useIndexedDB("debugRun");

  const handleDebugRunStart = useCallback(async () => {
    if (!currentProject) return;

    const engineReadyWorkflow = createEngineReadyWorkflow(
      currentProject.name,
      projectVariables,
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
            return {
              projectId: currentProject.id,
              jobId: data.job.id,
              status: data.job.status,
            };
          }
          return job;
        });
      } else {
        jobs.push({
          projectId: currentProject.id,
          jobId: data.job.id,
          status: data.job.status,
        });
      }
      await updateValue({ jobs });
      broadcastDebugRun(data.job.id); // NEW

      fitView({ duration: 400, padding: 0.5 });
    }
  }, [
    currentProject,
    projectVariables,
    rawWorkflows,
    broadcastDebugRun,
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

    const data = await useJobCancel(debugJob.jobId);
    if (data.isSuccess && currentProject?.id) {
      const jobs: JobState[] =
        debugRunState?.jobs?.filter((j) => j.projectId !== currentProject.id) ||
        [];
      await updateValue({ jobs });
      broadcastDebugRun(null);
    }
  }, [
    currentProject?.id,
    debugRunState?.jobs,
    updateValue,
    useJobCancel,
    broadcastDebugRun,
  ]);

  const loadExternalDebugJob = useCallback(
    async (jobId: string, userName: string) => {
      const jobs: JobState[] = debugRunState?.jobs || [];
      if (!currentProject) return;
      if (jobs.some((j) => j.jobId === jobId)) return;

      const newJobs = [
        ...jobs,
        {
          projectId: currentProject.id,
          jobId,
          status: "running" as JobState["status"],
        },
      ];
      await updateValue({ jobs: newJobs });

      toast({
        title: t("Now viewing {{userName}}'s debug run", {
          userName,
        }),
        description: t("You're now viewing {{userName}}'s debug session", {
          userName,
        }),
      });
    },
    [t, currentProject, debugRunState?.jobs, updateValue],
  );
  return {
    handleDebugRunStart,
    handleDebugRunStop,
    loadExternalDebugJob,
    activeDebugRuns,
  };
};
