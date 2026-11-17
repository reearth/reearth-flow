import { useReactFlow } from "@xyflow/react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { Awareness } from "y-protocols/awareness";

import { useProject, useWorkflowVariables } from "@flow/lib/gql";
import { useJob } from "@flow/lib/gql/job";
import { useT } from "@flow/lib/i18n";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useDebugAwareness } from "@flow/lib/yjs";
import type { JobState } from "@flow/stores";
import { useCurrentProject } from "@flow/stores";
import type { AnyWorkflowVariable, Node, Workflow } from "@flow/types";
import {
  carryCustomizedValues,
  missingRequiredVariables,
  toVariableDefaults,
} from "@flow/utils";
import type { VariableDefaults } from "@flow/utils";
import { createEngineReadyWorkflow } from "@flow/utils/toEngineWorkflow/engineReadyWorkflow";
import { pickIncrementalStart } from "@flow/utils/toEngineWorkflow/pickIncrementalStart";
import {
  buildRunGraph,
  lastRunCovers,
  runnableNodeIds,
  trimWorkflowsToNode,
} from "@flow/utils/toEngineWorkflow/trimToNode";

import { toast } from "../NotificationSystem/useToast";

export default ({
  rawWorkflows,
  yAwareness,
}: {
  rawWorkflows: Workflow[];
  yAwareness: Awareness;
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

  // The values the next run starts from, with the project defaults they were
  // built against, so a later default edit can tell which ones the user set.
  const [customDebugRun, setCustomDebugRun] = useState<
    { variables: AnyWorkflowVariable[]; defaults: VariableDefaults } | undefined
  >(undefined);
  const customDebugRunWorkflowVariables = customDebugRun?.variables;

  const { fitView } = useReactFlow();

  const { runProject } = useProject();
  const { useJobCancel } = useJob();

  const { value: debugRunState, updateValue } = useIndexedDB("debugRun");

  const debugJob = debugRunState?.jobs?.find(
    (job) => job.projectId === currentProject?.id,
  );

  const restoredProjectIdRef = useRef<string | undefined>(undefined);

  useEffect(() => {
    if (!workflowVariables || debugRunState === null) return;

    const isFreshProject = restoredProjectIdRef.current !== currentProject?.id;
    restoredProjectIdRef.current = currentProject?.id;

    setCustomDebugRun((prev) => {
      const baseline =
        isFreshProject || !prev
          ? debugJob?.variables && {
              variables: debugJob.variables,
              defaults: debugJob.variableDefaults,
            }
          : prev;
      const defaults = toVariableDefaults(workflowVariables);

      if (!baseline) return { variables: workflowVariables, defaults };
      // Only values the user set carry over; the rest follow the defaults.
      return {
        variables: carryCustomizedValues(
          workflowVariables,
          baseline.variables,
          baseline.defaults,
        ),
        defaults,
      };
    });
  }, [
    workflowVariables,
    debugRunState,
    debugJob?.variables,
    debugJob?.variableDefaults,
    currentProject?.id,
  ]);

  const runDebugWorkflow = useCallback(
    async ({
      previousJobId,
      startNodeId,
      endNodeId,
      overrideVariables,
    }: {
      previousJobId?: string;
      startNodeId?: string;
      endNodeId?: string;
      overrideVariables?: AnyWorkflowVariable[];
    } = {}) => {
      if (!currentProject) return;
      // The per-run copy is built once both the project's variables and the
      // saved debug state load; fall back to the project's variables so a
      // failed IndexedDB read doesn't block runs for good.
      const variablesForRun =
        overrideVariables ??
        customDebugRunWorkflowVariables ??
        workflowVariables;

      // Without the variables there is nothing to check required ones against,
      // and the run would go out with none set.
      if (!variablesForRun) {
        toast({
          title: t("Debug run not started"),
          description: t(
            "Workflow variables are still loading. Try again in a moment.",
          ),
          variant: "warning",
        });
        return;
      }

      // The variables dialog blocks this too, but every start path (including
      // re-running from a selected node) goes through here.
      const missing = missingRequiredVariables(variablesForRun);
      if (missing.length > 0) {
        toast({
          title: t("Debug run not started"),
          description: t(
            "These required workflow variables have no value: {{names}}",
            { names: missing.join(", ") },
          ),
          variant: "warning",
        });
        return;
      }
      let workflowsForRun = rawWorkflows;
      let reuse =
        previousJobId && startNodeId
          ? { previousJobId, startNodeId }
          : undefined;

      if (endNodeId) {
        const trimmed = trimWorkflowsToNode(rawWorkflows, endNodeId);
        if (!trimmed) return;
        workflowsForRun = trimmed.workflows;
        reuse = pickIncrementalStart({
          trimmedWorkflows: trimmed.workflows,
          selectedNodeId: endNodeId,
          lastRun: debugJob,
          variables: variablesForRun,
        });
      }

      const engineReadyWorkflow = createEngineReadyWorkflow(
        currentProject.name,
        variablesForRun,
        workflowsForRun,
      );

      if (!engineReadyWorkflow) return;

      const ranNodeIds = runnableNodeIds(workflowsForRun);

      const data = await runProject(
        currentProject.id,
        currentProject.workspaceId,
        engineReadyWorkflow,
        reuse?.previousJobId,
        reuse?.startNodeId,
        // Recorded on the job, so the run summary can show them to anyone
        // viewing the run.
        variablesForRun,
      );

      if (data.job) {
        // What the run's values are measured against. Without the project's
        // variables loaded the run used them as they were, so there is
        // nothing the user changed.
        const variableDefaults = toVariableDefaults(
          workflowVariables ?? variablesForRun,
        );
        if (overrideVariables) {
          setCustomDebugRun({
            variables: overrideVariables,
            defaults: variableDefaults,
          });
        }

        let jobs: JobState[] = debugRunState?.jobs || [];

        const jobState = (jobId: string, status: JobState["status"]) => ({
          projectId: currentProject.id,
          jobId,
          status,
          variables: variablesForRun,
          variableDefaults,
          startNodeId: reuse?.startNodeId,
          endNodeId,
          ranNodeIds,
        });

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
              return jobState(data.job.id, data.job.status);
            }
            return job;
          });
        } else {
          jobs.push(jobState(data.job.id, data.job.status));
        }
        await updateValue({ jobs });
        fitView({ duration: 400, padding: 0.5 });
      }
    },
    [
      t,
      currentProject,
      customDebugRunWorkflowVariables,
      workflowVariables,
      rawWorkflows,
      debugJob,
      debugRunState?.jobs,
      fitView,
      updateValue,
      runProject,
    ],
  );

  const handleDebugRunStart = useCallback(
    async (overrideVariables?: AnyWorkflowVariable[]) => {
      await runDebugWorkflow({ overrideVariables });
    },
    [runDebugWorkflow],
  );

  const handleFromSelectedNodeDebugRunStart = useCallback(
    async (node?: Node, nodes?: Node[]) => {
      const selectedNode = node ?? nodes?.[0];
      if (!selectedNode || !debugJob?.jobId) return;
      await runDebugWorkflow({
        previousJobId: debugJob.jobId,
        startNodeId: selectedNode.id,
      });
    },
    [runDebugWorkflow, debugJob?.jobId],
  );

  const handleToSelectedNodeDebugRunStart = useCallback(
    async (node?: Node, nodes?: Node[]) => {
      const selectedNode = node ?? nodes?.[0];
      if (!selectedNode) return;
      await runDebugWorkflow({ endNodeId: selectedNode.id });
    },
    [runDebugWorkflow],
  );

  const runGraph = useMemo(() => buildRunGraph(rawWorkflows), [rawWorkflows]);

  const canDebugRunFromNode = useCallback(
    (nodeId: string) =>
      !debugJob?.ranNodeIds ||
      lastRunCovers(runGraph, debugJob.ranNodeIds, nodeId),
    [runGraph, debugJob?.ranNodeIds],
  );

  const handleResetDebugRunWorkflowVariables = useCallback(() => {
    if (!workflowVariables) return;
    setCustomDebugRun({
      variables: workflowVariables,
      defaults: toVariableDefaults(workflowVariables),
    });
  }, [workflowVariables]);

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
      handleResetDebugRunWorkflowVariables();
    }
  }, [
    currentProject?.id,
    debugRunState?.jobs,
    updateValue,
    useJobCancel,
    handleResetDebugRunWorkflowVariables,
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
          joinedUserName: userName,
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

  useEffect(() => {
    broadcastDebugRun(
      debugJob?.jobId ?? null,
      debugJob?.status ? debugJob.status : undefined,
    );
  }, [debugJob?.jobId, debugJob?.status, broadcastDebugRun]);

  return {
    activeUsersDebugRuns,
    customDebugRunWorkflowVariables,
    workflowVariableDefaults: workflowVariables,
    refetchWorkflowVariables,
    handleDebugRunStart,
    handleFromSelectedNodeDebugRunStart,
    handleToSelectedNodeDebugRunStart,
    canDebugRunFromNode,
    handleDebugRunStop,
    handleResetDebugRunWorkflowVariables,
    loadExternalDebugJob,
  };
};
