import { useCallback, useEffect, useMemo, useRef } from "react";

import { useProject, useWorkflowVariables } from "@flow/lib/gql";
import { useIndexedDB } from "@flow/lib/indexedDB";
import { useCurrentProject } from "@flow/stores";
import type { ReaderSchemaProbeStatus } from "@flow/stores";
import type { Job, Node, NodeSchemaMeta, Workflow } from "@flow/types";
import { createEngineReadyWorkflow } from "@flow/utils/toEngineWorkflow/engineReadyWorkflow";

import { buildReaderAttributeSuggestions } from "./readerAttributeSuggestions";
import {
  fetchSchemaReport,
  findSchemaReportUrl,
  toNodeSchemaMeta,
} from "./schemaReport";

const PROBE_DEBOUNCE_MS = 600;

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

  const { value: previewSchemaState, updateValue } =
    useIndexedDB("previewSchema");

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
  // (Persisted probes are project-scoped, so they don't need clearing here.)
  useEffect(() => {
    debounceTimers.current.forEach((timer) => clearTimeout(timer));
    debounceTimers.current.clear();
    lastProbedParams.current.clear();
  }, [projectId]);

  const writeProbe = useCallback(
    (nodeId: string, jobId: string, status: ReaderSchemaProbeStatus) => {
      if (!projectId) return;
      if (status === "failed") lastProbedParams.current.delete(nodeId);
      void updateValue((prev) => {
        const probes = (prev.probes ?? []).filter(
          (probe) =>
            !(probe.projectId === projectId && probe.nodeId === nodeId),
        );
        probes.push({ projectId, nodeId, jobId, status });
        return { probes };
      });
    },
    [projectId, updateValue],
  );

  const clearProbe = useCallback(
    (nodeId: string) => {
      if (!projectId) return;
      void updateValue((prev) => ({
        probes: (prev.probes ?? []).filter(
          (probe) =>
            !(probe.projectId === projectId && probe.nodeId === nodeId),
        ),
      }));
    },
    [projectId, updateValue],
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
        writeProbe(nodeId, "", "failed");
        return;
      }
      writeProbe(nodeId, data.job.id, "running");
    },
    [currentProject, previewSchema, sampleSize, writeProbe],
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
        writeProbe(nodeId, job.id, "failed");
        return;
      }
      try {
        const report = await fetchSchemaReport(url);
        const nodeReport = report.nodes[nodeId];
        if (nodeReport) {
          onPersistSchemaRef.current(
            nodeId,
            toNodeSchemaMeta(
              nodeReport,
              report.sampleSize,
              new Date().toISOString(),
            ),
          );
        }
        clearProbe(nodeId);
      } catch {
        writeProbe(nodeId, job.id, "failed");
      }
    },
    [clearProbe, writeProbe],
  );

  const handleProbeError = useCallback(
    (nodeId: string) => writeProbe(nodeId, "", "failed"),
    [writeProbe],
  );

  const schemaProbes = useMemo(
    () =>
      (previewSchemaState?.probes ?? []).filter(
        (probe) => probe.projectId === projectId,
      ),
    [previewSchemaState, projectId],
  );

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
