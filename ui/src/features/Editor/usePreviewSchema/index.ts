import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { useProject, useWorkflowVariables } from "@flow/lib/gql";
import { useCurrentProject } from "@flow/stores";
import type { Job, Node, NodeSchemaMeta, Workflow } from "@flow/types";
import { createEngineReadyWorkflow } from "@flow/utils/toEngineWorkflow/engineReadyWorkflow";

import { probeSignature } from "./probeSignature";
import { buildReaderAttributeSuggestions } from "./readerAttributeSuggestions";
import {
  applySchemaPatch,
  fetchSchemaReport,
  findSchemaReportUrl,
  getNodeReportFailure,
  resolveNodeSchema,
  toNodeSchemaMeta,
} from "./schemaReport";
import type { SchemaProbeOutcomes } from "./schemaReport";

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
  onPersistSchema: (
    nodeId: string,
    patch: Partial<NodeSchemaMeta> | undefined,
  ) => void;
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

  const debounceTimers = useRef<Map<string, ReturnType<typeof setTimeout>>>(
    new Map(),
  );

  const lastProbedParams = useRef<Map<string, string>>(new Map());
  const pendingNodeIds = useRef<Set<string>>(new Set());
  const latestRequest = useRef<Map<string, number>>(new Map());

  // Probes this client started. Their outcome is part of the user's edit, so
  // this client writes it. Every other client watches the same probe
  // read-only and keeps the outcome locally: a client that merely has the
  // project open must not write, or the server records it as an edit.
  const startedJobIds = useRef<Set<string>>(new Set());
  const [outcomes, setOutcomes] = useState<SchemaProbeOutcomes>({});

  // The workflows with every locally worked-out outcome applied.
  const resolvedWorkflows = useMemo(() => {
    if (Object.keys(outcomes).length === 0) return rawWorkflows;
    return rawWorkflows.map((workflow) => ({
      ...workflow,
      nodes: workflow.nodes?.map((node) => {
        const schema = node.data?.nodeMetadata?.schema;
        const resolved = resolveNodeSchema(schema, outcomes);
        return resolved === schema
          ? node
          : {
              ...node,
              data: {
                ...node.data,
                nodeMetadata: { ...node.data.nodeMetadata, schema: resolved },
              },
            };
      }),
    }));
  }, [rawWorkflows, outcomes]);
  const resolvedWorkflowsRef = useRef(resolvedWorkflows);
  resolvedWorkflowsRef.current = resolvedWorkflows;

  // Records a probe's outcome: in the document if this client started it,
  // otherwise only here.
  const settleProbe = useCallback(
    (nodeId: string, jobId: string, patch: Partial<NodeSchemaMeta>) => {
      if (startedJobIds.current.delete(jobId)) {
        onPersistSchema(nodeId, patch);
        return;
      }
      const schema = rawWorkflowsRef.current
        .flatMap((workflow) => workflow.nodes ?? [])
        .find((n) => n.id === nodeId)?.data?.nodeMetadata?.schema;
      if (schema?.jobId !== jobId) return;
      const resolved = applySchemaPatch(schema, patch);
      if (!resolved) return;
      setOutcomes((prev) => ({ ...prev, [jobId]: resolved }));
    },
    [onPersistSchema],
  );

  useEffect(() => {
    const timers = debounceTimers.current;
    return () => {
      timers.forEach((timer) => clearTimeout(timer));
      timers.clear();
    };
  }, []);

  const markFailed = useCallback(
    (nodeId: string, note?: string) => {
      onPersistSchema(nodeId, { status: "failed", note });
    },
    [onPersistSchema],
  );

  const runProbe = useCallback(
    async (nodeId: string) => {
      if (!currentProject) return;
      if (!workflowVariablesRef.current) {
        pendingNodeIds.current.add(nodeId);
        return;
      }
      const engineReadyWorkflow = createEngineReadyWorkflow(
        currentProject.name,
        workflowVariablesRef.current,
        rawWorkflowsRef.current,
      );
      if (!engineReadyWorkflow) return;

      const request = (latestRequest.current.get(nodeId) ?? 0) + 1;
      latestRequest.current.set(nodeId, request);

      const data = await previewSchema(
        currentProject.id,
        currentProject.workspaceId,
        engineReadyWorkflow,
        sampleSize,
      );

      if (latestRequest.current.get(nodeId) !== request) return;

      if (!data.job?.id) {
        markFailed(nodeId);
        return;
      }
      startedJobIds.current.add(data.job.id);
      onPersistSchema(nodeId, {
        status: "running",
        jobId: data.job.id,
      });
    },
    [currentProject, previewSchema, sampleSize, markFailed, onPersistSchema],
  );

  const scheduleProbe = useCallback(
    (nodeId: string) => {
      const existing = debounceTimers.current.get(nodeId);
      if (existing) clearTimeout(existing);
      debounceTimers.current.set(
        nodeId,
        setTimeout(() => {
          debounceTimers.current.delete(nodeId);
          void runProbe(nodeId);
        }, PROBE_DEBOUNCE_MS),
      );
    },
    [runProbe],
  );

  const handleNodeParamsSaved = useCallback(
    (node: Node) => {
      if (node.type !== "reader") return;
      if (!node.data.params || Object.keys(node.data.params).length === 0) {
        return;
      }

      const signature = probeSignature(
        node.data.params,
        workflowVariablesRef.current,
      );
      const failed =
        resolvedWorkflowsRef.current
          .flatMap((workflow) => workflow.nodes ?? [])
          .find((n) => n.id === node.id)?.data?.nodeMetadata?.schema?.status ===
        "failed";
      if (!failed && lastProbedParams.current.get(node.id) === signature)
        return;
      lastProbedParams.current.set(node.id, signature);
      scheduleProbe(node.id);
    },
    [scheduleProbe],
  );

  useEffect(() => {
    if (!workflowVariables) return;
    const pending = pendingNodeIds.current;
    pendingNodeIds.current = new Set();
    for (const [nodeId, signature] of lastProbedParams.current) {
      const node = rawWorkflowsRef.current
        .flatMap((workflow) => workflow.nodes ?? [])
        .find((n) => n.id === nodeId);
      if (node?.type !== "reader") continue;
      const next = probeSignature(node.data.params, workflowVariables);
      if (next === signature && !pending.has(nodeId)) continue;
      lastProbedParams.current.set(nodeId, next);
      scheduleProbe(nodeId);
    }
  }, [workflowVariables, scheduleProbe]);

  const handleProbeComplete = useCallback(
    async (nodeId: string, job: Job) => {
      const fail = (note?: string) =>
        settleProbe(nodeId, job.id, { status: "failed", note });
      const url = findSchemaReportUrl(job.outputURLs);
      if (!url) {
        fail();
        return;
      }
      try {
        const report = await fetchSchemaReport(url);
        const nodeReport = report.nodes[nodeId];
        // The job produced a report but it doesn't cover this node.
        if (!nodeReport) {
          fail();
          return;
        }
        // The job completes even when the reader couldn't sample its data; the
        // engine only flags that with a note + open/empty port. Treat that as a
        // failure (surfacing the reason) rather than a hollow "complete".
        const failureNote = getNodeReportFailure(nodeReport);
        if (failureNote) {
          fail(failureNote);
          return;
        }
        settleProbe(
          nodeId,
          job.id,
          toNodeSchemaMeta(
            nodeReport,
            report.sampleSize,
            new Date().toISOString(),
          ),
        );
      } catch {
        fail();
      }
    },
    [settleProbe],
  );

  const handleProbeError = useCallback(
    (nodeId: string, jobId: string) =>
      settleProbe(nodeId, jobId, { status: "failed" }),
    [settleProbe],
  );

  // Every probe still running in the document, whoever started it, except the
  // ones this client has already worked out.
  const schemaProbes = useMemo<RunningSchemaProbe[]>(() => {
    const probes: RunningSchemaProbe[] = [];
    for (const workflow of resolvedWorkflows) {
      for (const node of workflow.nodes ?? []) {
        const schema = node.data?.nodeMetadata?.schema;
        if (schema?.status === "running" && schema.jobId) {
          probes.push({ nodeId: node.id, jobId: schema.jobId });
        }
      }
    }
    return probes;
  }, [resolvedWorkflows]);

  // `rawWorkflows` is rebuilt from the yjs doc on every render, so this memo
  // always recomputes. Key on the resulting attribute list instead, so
  // consumers see a stable identity while the attributes themselves are
  // unchanged — otherwise every unrelated Editor render resets the FlowExpr
  // autocomplete.
  const computedAttributeSuggestions = useMemo(
    () => buildReaderAttributeSuggestions(resolvedWorkflows, openNodeId),
    [resolvedWorkflows, openNodeId],
  );
  const attributeSuggestionsKey = computedAttributeSuggestions
    .map((suggestion) => `${suggestion.label}:${suggestion.detail ?? ""}`)
    .join("|");
  const readerAttributeSuggestions = useMemo(
    () => computedAttributeSuggestions,
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [attributeSuggestionsKey],
  );

  return {
    schemaProbes,
    schemaProbeOutcomes: outcomes,
    readerAttributeSuggestions,
    handleNodeParamsSaved,
    handleProbeComplete,
    handleProbeError,
  };
};
