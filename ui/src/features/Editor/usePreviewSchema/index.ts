import { useCallback, useEffect, useMemo, useRef } from "react";

import { useProject, useWorkflowVariables } from "@flow/lib/gql";
import { useCurrentProject, useReaderSchemaProbes } from "@flow/stores";
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
  const { previewSchema } = useProject();
  const [probes, setProbes] = useReaderSchemaProbes();

  const { useGetWorkflowVariables } = useWorkflowVariables();
  const { workflowVariables } = useGetWorkflowVariables(
    currentProject?.id ?? "",
  );

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

  useEffect(() => {
    const timers = debounceTimers.current;
    return () => {
      timers.forEach((timer) => clearTimeout(timer));
      timers.clear();
    };
  }, []);

  const setProbeStatus = useCallback(
    (nodeId: string, jobId: string, status: "running" | "failed") =>
      setProbes((prev) => ({ ...prev, [nodeId]: { nodeId, jobId, status } })),
    [setProbes],
  );

  const clearProbe = useCallback(
    (nodeId: string) =>
      setProbes((prev) => {
        if (!prev[nodeId]) return prev;
        const { [nodeId]: _removed, ...rest } = prev;
        return rest;
      }),
    [setProbes],
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

      // Immediate feedback while the mutation is in flight (jobId filled in next).
      setProbeStatus(nodeId, "", "running");

      const data = await previewSchema(
        currentProject.id,
        currentProject.workspaceId,
        engineReadyWorkflow,
        sampleSize,
      );

      if (!data.job?.id) {
        setProbeStatus(nodeId, "", "failed");
        return;
      }
      setProbeStatus(nodeId, data.job.id, "running");
    },
    [currentProject, previewSchema, sampleSize, setProbeStatus],
  );

  const handleNodeParamsSaved = useCallback(
    (node: Node) => {
      if (node.type !== "reader") return;
      if (!node.data.params || Object.keys(node.data.params).length === 0) {
        return;
      }
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
        setProbeStatus(nodeId, job.id, "failed");
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
        setProbeStatus(nodeId, job.id, "failed");
      }
    },
    [clearProbe, setProbeStatus],
  );

  const handleProbeError = useCallback(
    (nodeId: string) => {
      setProbes((prev) => {
        const existing = prev[nodeId];
        return {
          ...prev,
          [nodeId]: { nodeId, jobId: existing?.jobId ?? "", status: "failed" },
        };
      });
    },
    [setProbes],
  );

  const readerAttributeSuggestions = useMemo(
    () => buildReaderAttributeSuggestions(rawWorkflows, openNodeId),
    [rawWorkflows, openNodeId],
  );

  return {
    schemaProbes: probes,
    readerAttributeSuggestions,
    handleNodeParamsSaved,
    handleProbeComplete,
    handleProbeError,
  };
};
