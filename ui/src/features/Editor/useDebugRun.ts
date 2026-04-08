import { useNavigate } from "@tanstack/react-router";
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
  const navigate = useNavigate();

  const [currentJobId, setCurrentJobId] = useState<string | undefined>(
    undefined,
  );

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
        return;
      }
      const data = await runProject(
        currentProject.id,
        currentProject.workspaceId,
        engineReadyWorkflow,
        jobId,
        selectedNodeId,
      );

      if (data.job?.id) {
        const newJobId = data.job.id;
        setCurrentJobId(newJobId);

        // Write to IndexedDB so the debug route can find intermediate data etc.
        const existing = debugRunState?.jobs || [];
        const jobs: JobState[] = [
          ...existing.filter((j) => j.jobId !== newJobId),
          {
            projectId: currentProject.id,
            jobId: newJobId,
            status: data.job.status,
          },
        ];
        await updateValue({ jobs });

        navigate({
          to: `/workspaces/${currentProject.workspaceId}/projects/${currentProject.id}/debug/${newJobId}`,
        });
        fitView({ duration: 400, padding: 0.5 });
      }
    },
    [
      currentProject,
      customDebugRunWorkflowVariables,
      rawWorkflows,
      debugRunState?.jobs,
      fitView,
      updateValue,
      navigate,
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
      if (!selectedNode || !currentJobId) return;
      await runDebugWorkflow(currentJobId, selectedNode.id);
    },
    [runDebugWorkflow, currentJobId],
  );

  const handleDebugRunStop = useCallback(async () => {
    if (!currentJobId) return;
    const data = await useJobCancel(currentJobId);
    if (data.isSuccess) {
      setCurrentJobId(undefined);
    }
  }, [currentJobId, useJobCancel]);

  const loadExternalDebugJob = useCallback(
    (jobId: string, userName: string) => {
      if (!currentProject) return;
      navigate({
        to: `/workspaces/${currentProject.workspaceId}/projects/${currentProject.id}/debug/${jobId}`,
      });
      toast({
        title: t("Now viewing {{userName}}'s debug run", { userName }),
        description: t("You're now viewing {{userName}}'s debug session", {
          userName,
        }),
      });
    },
    [t, currentProject, navigate],
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

  useEffect(() => {
    broadcastDebugRun(currentJobId ?? null, undefined);
  }, [currentJobId, broadcastDebugRun]);

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
