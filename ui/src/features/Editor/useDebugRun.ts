import { useReactFlow } from "@xyflow/react";
import { useCallback, useEffect, useState } from "react";
import type { Awareness } from "y-protocols/awareness";

import { useProject, useWorkflowVariables } from "@flow/lib/gql";
import { useJob } from "@flow/lib/gql/job";
import { useT } from "@flow/lib/i18n";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useDebugAwareness } from "@flow/lib/yjs";
import { JobState, useCurrentProject } from "@flow/stores";
import type { AnyWorkflowVariable, Node, Workflow } from "@flow/types";
import { createEngineReadyWorkflow } from "@flow/utils/toEngineWorkflow/engineReadyWorkflow";

import { toast } from "../NotificationSystem/useToast";

export default ({
  rawWorkflows,
  yAwareness,
  onProjectSnapshotSave,
}: {
  rawWorkflows: Workflow[];
  yAwareness: Awareness;
  onProjectSnapshotSave: () => Promise<void>;
}) => {
  const t = useT();
  const [currentProject] = useCurrentProject();

  const { activeUsersDebugRuns, broadcastDebugRun } = useDebugAwareness({
    yAwareness,
    projectId: currentProject?.id,
  });

  const { useGetWorkflowVariables } = useWorkflowVariables();
  const { workflowVariables, refetch: refetchWorkflowVariables } =
    useGetWorkflowVariables(currentProject?.id ?? "");

  const [customDebugRunWorkflowVariables, setCustomDebugRunWorkflowVariables] =
    useState<AnyWorkflowVariable[] | undefined>(undefined);

  useEffect(() => {
    if (!workflowVariables) return;
    setCustomDebugRunWorkflowVariables((prev) => {
      if (!prev) return workflowVariables;
      return workflowVariables.map((workflowVariable) => {
        const existingCustom = prev.find(
          (customVariable) =>
            customVariable.name === workflowVariable.name &&
            customVariable.type === workflowVariable.type &&
            JSON.stringify(customVariable.defaultValue) ===
              JSON.stringify(workflowVariable.defaultValue),
        );
        return existingCustom || workflowVariable;
      });
    });
  }, [workflowVariables]);

  const { fitView } = useReactFlow();

  const { runProject } = useProject();
  const { useJobCancel } = useJob();

  const { value: debugRunState, updateValue } = useIndexedDB("debugRun");

  const debugJob = debugRunState?.jobs?.find(
    (job) => job.projectId === currentProject?.id,
  );

  const runDebugWorkflow = useCallback(
    async (jobId?: string, selectedNodeId?: string) => {
      if (!currentProject) return;
      const engineReadyWorkflow = createEngineReadyWorkflow(
        currentProject.name,
        customDebugRunWorkflowVariables,
        rawWorkflows,
      );

      if (!engineReadyWorkflow) return;
      try {
        await onProjectSnapshotSave();
      } catch {
        // Abort run if snapshot save fails
        return;
      }
      const data = await runProject(
        currentProject.id,
        currentProject.workspaceId,
        engineReadyWorkflow,
        jobId,
        selectedNodeId,
      );

      if (data.job) {
        let jobs: JobState[] = debugRunState?.jobs || [];

        if (!data.job.id) {
          jobs =
            debugRunState?.jobs?.filter(
              (job) => job.projectId !== currentProject.id,
            ) || [];
        } else if (
          debugRunState?.jobs?.some(
            (job) => job.projectId === currentProject.id,
          )
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
        broadcastDebugRun(data.job.id, data.job.status);

        fitView({ duration: 400, padding: 0.5 });
      }
    },
    [
      currentProject,
      customDebugRunWorkflowVariables,
      rawWorkflows,
      broadcastDebugRun,
      debugRunState?.jobs,
      fitView,
      updateValue,
      runProject,
      onProjectSnapshotSave,
    ],
  );

  const handleDebugRunStart = useCallback(async () => {
    await runDebugWorkflow();
  }, [runDebugWorkflow]);

  const handleFromSelectedNodeDebugRunStart = useCallback(
    async (node?: Node, nodes?: Node[]) => {
      const selectedNode = node ?? nodes?.[0];
      if (!selectedNode || !debugJob?.jobId) return;
      await runDebugWorkflow(debugJob.jobId, selectedNode.id);
    },
    [runDebugWorkflow, debugJob?.jobId],
  );

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
      if (!currentProject || !debugRunState) return;

      // Check if job already exists before updating
      const existingJobs = debugRunState?.jobs || [];
      if (existingJobs.some((j) => j.jobId === jobId)) {
        return; // Already viewing this job, so no need to update
      }

      // Clear any existing debug run that the user has run
      const filteredJobs = existingJobs.filter(
        (job) => job.projectId !== currentProject.id,
      );

      const states = Array.from(yAwareness.getStates());
      const debugJobStatus = states.find(([, state]) =>
        state.debugRun ? state.debugRun.jobId === jobId : false,
      )?.[1]?.debugRun?.status;

      // Add the new external debug job
      const newJobs = [
        ...filteredJobs,
        {
          projectId: currentProject.id,
          jobId,
          status: debugJobStatus,
        },
      ];

      await updateValue({ jobs: newJobs });

      // Show toast after successful update
      toast({
        title: t("Now viewing {{userName}}'s debug run", {
          userName,
        }),
        description: t("You're now viewing {{userName}}'s debug session", {
          userName,
        }),
      });
    },
    [t, currentProject, debugRunState, updateValue, yAwareness],
  );

  const handleDebugRunVariableValueChange = useCallback(
    (index: number, newValue: any) => {
      setCustomDebugRunWorkflowVariables((prev) =>
        prev?.map((variable, i) =>
          i === index ? { ...variable, defaultValue: newValue } : variable,
        ),
      );
    },
    [],
  );

  return {
    activeUsersDebugRuns,
    customDebugRunWorkflowVariables,
    refetchWorkflowVariables,
    handleDebugRunStart,
    handleFromSelectedNodeDebugRunStart,
    handleDebugRunStop,
    handleDebugRunVariableValueChange,
    loadExternalDebugJob,
  };
};
