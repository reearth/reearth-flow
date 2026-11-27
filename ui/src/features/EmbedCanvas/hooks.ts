import { useReactFlow } from "@xyflow/react";
import type { MouseEvent } from "react";
import { useCallback, useEffect, useMemo, useState } from "react";

import { ProjectCorruptionError } from "@flow/errors";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import type {
  EmbedErrorReason,
  EmbedInboundMessage,
  EmbedWorkflowPathItem,
} from "@flow/lib/embedBridge";
import { useEmbedBridge } from "@flow/lib/embedBridge";
import { useDocument } from "@flow/lib/gql/document/useApi";
import type { Node, Workflow } from "@flow/types";
import { buildWorkflowParentMap, getWorkflowLineage } from "@flow/utils";
import { toFinitePosition } from "@flow/utils/toFinitePosition";

import { snapshotToWorkflows } from "./snapshotToWorkflows";

type LoadedWorkflows =
  | { workflows: Workflow[]; error?: undefined }
  | { workflows?: undefined; error: EmbedErrorReason };

const loadWorkflows = (updates: number[]): LoadedWorkflows => {
  try {
    return { workflows: snapshotToWorkflows(updates) };
  } catch (error) {
    return {
      error:
        error instanceof ProjectCorruptionError ? "corrupted" : "unavailable",
    };
  }
};

export default ({ projectId }: { projectId: string }) => {
  const { fitView } = useReactFlow();
  const { useGetLatestProjectSnapshotOnce } = useDocument();
  const { projectDocument, isLoading, isError } =
    useGetLatestProjectSnapshotOnce(projectId);

  const loaded = useMemo((): LoadedWorkflows | undefined => {
    if (isError || projectDocument === null) return { error: "unavailable" };
    if (!projectDocument) return;
    return loadWorkflows(projectDocument.updates);
  }, [projectDocument, isError]);

  const workflows = loaded?.workflows;
  const error = loaded?.error;

  const workflowsById = useMemo(
    () => new Map((workflows ?? []).map((w) => [w.id, w])),
    [workflows],
  );

  const parentMap = useMemo(
    () => buildWorkflowParentMap(workflows ?? []),
    [workflows],
  );

  const [currentWorkflowId, setCurrentWorkflowId] = useState(
    DEFAULT_ENTRY_GRAPH_ID,
  );
  const [openNode, setOpenNode] = useState<Node | undefined>();

  const currentWorkflow = workflowsById.get(currentWorkflowId);
  const isMainWorkflow = currentWorkflowId === DEFAULT_ENTRY_GRAPH_ID;

  const nodes = useMemo(
    () =>
      (currentWorkflow?.nodes ?? []).map((node) => ({
        ...node,
        position: toFinitePosition(node.position),
        selected: false,
      })),
    [currentWorkflow],
  );

  const edges = useMemo(() => currentWorkflow?.edges ?? [], [currentWorkflow]);

  const path: EmbedWorkflowPathItem[] = useMemo(
    () =>
      workflows
        ? getWorkflowLineage(parentMap, currentWorkflowId).map((id) => ({
            id,
            name: workflowsById.get(id)?.name ?? "",
          }))
        : [],
    [workflows, parentMap, currentWorkflowId, workflowsById],
  );

  const handleWorkflowOpen = useCallback(
    (workflowId: string) => {
      if (!workflowsById.has(workflowId)) return;
      setOpenNode(undefined);
      setCurrentWorkflowId(workflowId);
    },
    [workflowsById],
  );

  const handleBridgeMessage = useCallback(
    (message: EmbedInboundMessage) => {
      if (message.type === "flow:open-workflow")
        handleWorkflowOpen(message.workflowId);
    },
    [handleWorkflowOpen],
  );

  const { post } = useEmbedBridge(handleBridgeMessage);

  useEffect(() => {
    if (error) post({ type: "flow:error", reason: error });
  }, [error, post]);

  useEffect(() => {
    if (!workflows || !projectDocument) return;
    post({
      type: "flow:loaded",
      projectId,
      timestamp: projectDocument.timestamp,
      isEmpty: !workflowsById.get(DEFAULT_ENTRY_GRAPH_ID)?.nodes?.length,
    });
  }, [workflows, workflowsById, projectDocument, projectId, post]);

  useEffect(() => {
    if (path.length) post({ type: "flow:workflow-changed", path });
  }, [path, post]);

  useEffect(() => {
    if (!nodes.length) return;
    const frame = requestAnimationFrame(() =>
      fitView({ padding: 0.5, maxZoom: 1 }),
    );
    return () => cancelAnimationFrame(frame);
  }, [nodes, fitView]);

  const handleOpenNode = useCallback(
    (nodeId?: string) => {
      setOpenNode((current) =>
        !nodeId || current?.id === nodeId
          ? undefined
          : nodes.find((n) => n.id === nodeId),
      );
    },
    [nodes],
  );

  const handleNodeSettings = useCallback(
    (_e: MouseEvent | undefined, nodeId: string) => handleOpenNode(nodeId),
    [handleOpenNode],
  );

  return {
    nodes,
    edges,
    error,
    isLoading: isLoading || (!loaded && !isError),
    isEmpty: !!workflows && !nodes.length,
    isMainWorkflow,
    openNode,
    handleWorkflowOpen,
    handleOpenNode,
    handleNodeSettings,
  };
};
