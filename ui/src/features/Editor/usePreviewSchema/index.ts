import { useCallback, useEffect, useMemo, useRef } from "react";

import { useProject, useWorkflowVariables } from "@flow/lib/gql";
import { useCurrentProject } from "@flow/stores";
import type { Job, Node, NodeSchemaMeta, Workflow } from "@flow/types";
import { createEngineReadyWorkflow } from "@flow/utils/toEngineWorkflow/engineReadyWorkflow";

import { buildReaderAttributeSuggestions } from "./readerAttributeSuggestions";
import {
  fetchSchemaReport,
  findSchemaReportUrl,
  getNodeReportFailure,
  toNodeSchemaMeta,
} from "./schemaReport";

const PROBE_DEBOUNCE_MS = 600;

export type RunningSchemaProbe = {
  nodeId: string;
  jobId: string;
};

export default ({
  rawWorkflows,
  openNodeId,
  onPersistSchema,
  sampleSize,
}: {
  rawWorkflows: Workflow[];
  openNodeId?: string;
  onPersistSchema: (nodeId: string, schema: NodeSchemaMeta | undefined) => void;
  sampleSize?: number;
}) => {
  const [currentProject] = useCurrentProject();
  const projectId = currentProject?.id;
  const { previewSchema } = useProject();

  const { useGetWorkflowVariables } = useWorkflowVariables();
  const { workflowVariables } = useGetWorkflowVariables(projectId ?? "");

  // Refs so the debounced probe reads the freshest workflow/variables, not the
  // values captured when the save fired.
  const rawWorkflowsRef = useRef(rawWorkflows);
  rawWorkflowsRef.current = rawWorkflows;
  const workflowVariablesRef = useRef(workflowVariables);
  workflowVariablesRef.current = workflowVariables;
  const onPersistSchemaRef = useRef(onPersistSchema);
  onPersistSchemaRef.current = onPersistSchema;

  const debounceTimers = useRef<Map<string, ReturnType<typeof setTimeout>>>(
    new Map(),
  );

  const lastProbedParams = useRef<Map<string, string>>(new Map());

  useEffect(() => {
    const timers = debounceTimers.current;
    return () => {
      timers.forEach((timer) => clearTimeout(timer));
      timers.clear();
    };
  }, []);

  // In-memory bookkeeping is per-project; clear it when the project changes.
  useEffect(() => {
    debounceTimers.current.forEach((timer) => clearTimeout(timer));
    debounceTimers.current.clear();
    lastProbedParams.current.clear();
  }, [projectId]);

  const getNodeSchema = useCallback(
    (nodeId: string): NodeSchemaMeta | undefined => {
      for (const workflow of rawWorkflowsRef.current) {
        const node = workflow.nodes?.find((n) => n.id === nodeId);
        if (node) return node.data?.nodeMetadata?.schema;
      }
      return undefined;
    },
    [],
  );

  const persistStatus = useCallback(
    (
      nodeId: string,
      status: "running" | "failed",
      extra?: { jobId?: string; note?: string },
    ) => {
      const existing = getNodeSchema(nodeId);
      onPersistSchemaRef.current(nodeId, {
        ports: existing?.ports ?? {},
        sampleSize: existing?.sampleSize,
        sampledAt: existing?.sampledAt,
        // On failure surface the engine's reason (for debugging); while running
        // keep whatever note was previously shown.
        note: status === "failed" ? extra?.note : existing?.note,
        status,
        ...(extra?.jobId ? { jobId: extra.jobId } : {}),
      });
    },
    [getNodeSchema],
  );

  const markFailed = useCallback(
    (nodeId: string, note?: string) => {
      lastProbedParams.current.delete(nodeId);
      persistStatus(nodeId, "failed", { note });
    },
    [persistStatus],
  );

  const runProbe = useCallback(
    async (nodeId: string) => {
      if (!currentProject) return;
      const engineReadyWorkflow = createEngineReadyWorkflow(
        currentProject.name,
        workflowVariablesRef.current,
        rawWorkflowsRef.current,
      );
      if (!engineReadyWorkflow) return;

      const data = await previewSchema(
        currentProject.id,
        currentProject.workspaceId,
        engineReadyWorkflow,
        sampleSize,
      );

      if (!data.job?.id) {
        markFailed(nodeId);
        return;
      }
      persistStatus(nodeId, "running", { jobId: data.job.id });
    },
    [currentProject, previewSchema, sampleSize, persistStatus, markFailed],
  );

  const handleNodeParamsSaved = useCallback(
    (node: Node) => {
      if (node.type !== "reader") return;
      if (!node.data.params || Object.keys(node.data.params).length === 0) {
        return;
      }

      const signature = JSON.stringify(node.data.params);
      if (lastProbedParams.current.get(node.id) === signature) return;
      lastProbedParams.current.set(node.id, signature);

      const existing = debounceTimers.current.get(node.id);
      if (existing) clearTimeout(existing);
      debounceTimers.current.set(
        node.id,
        setTimeout(() => {
          debounceTimers.current.delete(node.id);
          void runProbe(node.id);
        }, PROBE_DEBOUNCE_MS),
      );
    },
    [runProbe],
  );

  const handleProbeComplete = useCallback(
    async (nodeId: string, job: Job) => {
      const url = findSchemaReportUrl(job.outputURLs);
      if (!url) {
        markFailed(nodeId);
        return;
      }
      try {
        const report = await fetchSchemaReport(url);
        const nodeReport = report.nodes[nodeId];
        // The job produced a report but it doesn't cover this node.
        if (!nodeReport) {
          markFailed(nodeId);
          return;
        }
        // The job completes even when the reader couldn't sample its data; the
        // engine only flags that with a note + open/empty port. Treat that as a
        // failure (surfacing the reason) rather than a hollow "complete".
        const failureNote = getNodeReportFailure(nodeReport);
        if (failureNote) {
          markFailed(nodeId, failureNote);
          return;
        }
        onPersistSchemaRef.current(
          nodeId,
          toNodeSchemaMeta(
            nodeReport,
            report.sampleSize,
            new Date().toISOString(),
          ),
        );
      } catch {
        markFailed(nodeId);
      }
    },
    [markFailed],
  );

  const handleProbeError = useCallback(
    (nodeId: string) => markFailed(nodeId),
    [markFailed],
  );

  const schemaProbes = useMemo<RunningSchemaProbe[]>(() => {
    const probes: RunningSchemaProbe[] = [];
    for (const workflow of rawWorkflows) {
      for (const node of workflow.nodes ?? []) {
        const schema = node.data?.nodeMetadata?.schema;
        if (schema?.status === "running" && schema.jobId) {
          probes.push({ nodeId: node.id, jobId: schema.jobId });
        }
      }
    }
    return probes;
  }, [rawWorkflows]);

  useEffect(() => {
    for (const workflow of rawWorkflows) {
      for (const node of workflow.nodes ?? []) {
        if (node.data?.nodeMetadata?.schema?.status === "failed") {
          lastProbedParams.current.delete(node.id);
        }
      }
    }
  }, [rawWorkflows]);

  const readerAttributeSuggestions = useMemo(
    () => buildReaderAttributeSuggestions(rawWorkflows, openNodeId),
    [rawWorkflows, openNodeId],
  );

  return {
    schemaProbes,
    readerAttributeSuggestions,
    handleNodeParamsSaved,
    handleProbeComplete,
    handleProbeError,
  };
};
