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

/** Debounce window so a burst of saves on one reader triggers a single probe. */
const PROBE_DEBOUNCE_MS = 600;

export default ({
  rawWorkflows,
  onPersistSchema,
  sampleSize,
}: {
  rawWorkflows: Workflow[];
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

  // Keep the latest workflow/variables in refs so the debounced probe always
  // builds the freshest engine-ready workflow (the Yjs save that triggers it
  // lands a render before the timer fires).
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
      if (!currentProject) {
        console.debug("[previewSchema] runProbe skipped: no current project", {
          nodeId,
        });
        return;
      }
      const engineReadyWorkflow = createEngineReadyWorkflow(
        currentProject.name,
        workflowVariablesRef.current,
        rawWorkflowsRef.current,
      );
      if (!engineReadyWorkflow) {
        console.debug(
          "[previewSchema] runProbe skipped: engine-ready workflow could not be built",
          { nodeId, workflowCount: rawWorkflowsRef.current?.length },
        );
        return;
      }

      // Immediate feedback while the mutation is in flight (jobId filled in next).
      setProbeStatus(nodeId, "", "running");

      console.debug("[previewSchema] firing previewSchema mutation", {
        nodeId,
        projectId: currentProject.id,
        sampleSize,
      });

      const data = await previewSchema(
        currentProject.id,
        currentProject.workspaceId,
        engineReadyWorkflow,
        sampleSize,
      );

      if (!data.job?.id) {
        console.debug(
          "[previewSchema] mutation returned no job (request failed)",
          { nodeId },
        );
        setProbeStatus(nodeId, "", "failed");
        return;
      }
      console.debug("[previewSchema] mutation succeeded", {
        nodeId,
        jobId: data.job.id,
        status: data.job.status,
      });
      setProbeStatus(nodeId, data.job.id, "running");
    },
    [currentProject, previewSchema, sampleSize, setProbeStatus],
  );

  /**
   * Called when a node's params are saved. Only readers (with configured
   * params) trigger a probe; the call is debounced per node.
   */
  const handleNodeParamsSaved = useCallback(
    (node: Node) => {
      console.debug("[previewSchema] node params saved", {
        nodeId: node.id,
        type: node.type,
        paramKeys: node.data.params ? Object.keys(node.data.params) : [],
      });
      if (node.type !== "reader") {
        console.debug("[previewSchema] skipped: node is not a reader", {
          nodeId: node.id,
          type: node.type,
        });
        return;
      }
      if (!node.data.params || Object.keys(node.data.params).length === 0) {
        console.debug("[previewSchema] skipped: reader has no params set", {
          nodeId: node.id,
        });
        return;
      }
      console.debug(
        `[previewSchema] scheduling probe in ${PROBE_DEBOUNCE_MS}ms`,
        { nodeId: node.id },
      );
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
      console.debug("[previewSchema] job completed", {
        nodeId,
        jobId: job.id,
        outputURLs: job.outputURLs,
      });
      const url = findSchemaReportUrl(job.outputURLs);
      if (!url) {
        console.debug(
          "[previewSchema] no schema-report URL in outputURLs → failed",
          { nodeId },
        );
        setProbeStatus(nodeId, job.id, "failed");
        return;
      }
      try {
        const report = await fetchSchemaReport(url);
        const nodeReport = report.nodes[nodeId];
        console.debug("[previewSchema] fetched report", {
          nodeId,
          url,
          nodeFound: !!nodeReport,
          nodeIdsInReport: Object.keys(report.nodes),
        });
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
      } catch (err) {
        console.debug("[previewSchema] failed to fetch/parse report", {
          nodeId,
          url,
          err,
        });
        setProbeStatus(nodeId, job.id, "failed");
      }
    },
    [clearProbe, setProbeStatus],
  );

  const handleProbeError = useCallback(
    (nodeId: string) => {
      console.debug("[previewSchema] job failed/cancelled", { nodeId });
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
    () => buildReaderAttributeSuggestions(rawWorkflows),
    [rawWorkflows],
  );

  return {
    schemaProbes: probes,
    readerAttributeSuggestions,
    handleNodeParamsSaved,
    handleProbeComplete,
    handleProbeError,
  };
};
