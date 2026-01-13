import { NodeChange, useReactFlow } from "@xyflow/react";
import { useMemo, useCallback, useState, useRef } from "react";

import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { useDoubleClick } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";
import { Node, Workflow } from "@flow/types";
import { useCurrentProject } from "@flow/stores";
import { useWorkflowVariables } from "@flow/lib/gql";

export type SearchNodeResult = {
  id: string;
  displayName: string; // customName || officialName
  officialName: string;
  customName?: string;
  workflowId: string;
  workflowName: string;
  isMainWorkflow: boolean;
  nodeType: string;
  content?: string;
  params?: Record<string, any>;
};

const checkParamsContainWorkflowVariableNames = (
  params: Record<string, any> | undefined,
  variableNames: string[],
): boolean => {
  if (!params || variableNames.length === 0) return false;

  const checkValue = (value: any): boolean => {
    if (typeof value === "string") {
      return variableNames.some((variableName) => {
        const patterns = [
          new RegExp(`env\\.get\\(["']${variableName}["']\\)`, "i"),
          new RegExp(`env\\.get\\(['"]${variableName}['"]\\)`, "i"),
        ];
        return patterns.some((pattern) => pattern.test(value));
      });
    } else if (Array.isArray(value)) {
      return value.some((item) => checkValue(item));
    } else if (value !== null && typeof value === "object") {
      return Object.values(value).some((val) => checkValue(val));
    }
    return false;
  };

  return checkValue(params);
};

export default ({
  rawWorkflows,
  currentWorkflowId,
  onNodesChange,
  onWorkflowOpen,
}: {
  rawWorkflows: Workflow[];
  currentWorkflowId: string;
  onNodesChange?: (changes: NodeChange<Node>[]) => void;
  onWorkflowOpen: (id: string) => void;
}) => {
  const t = useT();
  const { setCenter, getNode } = useReactFlow();
  const prevSelectedNodeIdRef = useRef<string | null>(null);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [searchTerm, setSearchTerm] = useState<string>("");
  const [currentProject] = useCurrentProject();
  const { useGetWorkflowVariables } = useWorkflowVariables();
  const { workflowVariables } = useGetWorkflowVariables(
    currentProject?.id ?? "",
  );

  const actionTypes = useMemo(() => {
    return [
      {
        value: "all",
        label: t("All Actions"),
      },
      {
        value: "reader",
        label: t("Readers"),
      },
      {
        value: "transformer",
        label: t("Transformers"),
      },
      {
        value: "writer",
        label: t("Writers"),
      },
      {
        value: "subworkflow",
        label: t("Subworkflows"),
      },
      {
        value: "note",
        label: t("Notes"),
      },
      {
        value: "batch",
        label: t("Batches"),
      },
    ];
  }, [t]);

  const [currentActionTypeFilter, setCurrentActionTypeFilter] = useState("all");
  const [currentWorkflowFilter, setCurrentWorkflowFilter] = useState("all");

  const allNodes: SearchNodeResult[] = useMemo(() => {
    return rawWorkflows.flatMap((workflow) =>
      (workflow.nodes || []).map((node) => ({
        id: node.id,
        displayName:
          node.data.customizations?.customName || node.data.officialName,
        officialName: node.data.officialName,
        customName: node.data.customizations?.customName,
        workflowId: workflow.id,
        workflowName: workflow.name || "Unnamed Workflow",
        isMainWorkflow: workflow.id === DEFAULT_ENTRY_GRAPH_ID,
        nodeType: node.type || "default",
        content: node.data.customizations?.content,
        params: node.data.params,
      })),
    );
  }, [rawWorkflows]);

  const workflows = useMemo(
    () => [
      {
        value: "all",
        label: t("All Workflows"),
      },
      ...rawWorkflows.map((wf) => ({
        value: wf.id,
        label: wf.name || "Unnamed Workflow",
      })),
    ],
    [rawWorkflows, t],
  );

  const filteredNodes: SearchNodeResult[] = useMemo(() => {
    const matchingVariableNames =
      workflowVariables
        ?.filter((variable) =>
          variable.name.toLowerCase().includes(searchTerm.toLowerCase()),
        )
        .map((variable) => variable.name) || [];

    return allNodes.filter((node) => {
      const matchesSearchTerm =
        node.displayName.toLowerCase().includes(searchTerm.toLowerCase()) ||
        node.content?.toLowerCase().includes(searchTerm.toLowerCase()) ||
        checkParamsContainWorkflowVariableNames(
          node.params,
          matchingVariableNames,
        );
      const matchesActionType =
        currentActionTypeFilter === "all" ||
        node.nodeType === currentActionTypeFilter;
      const matchesWorkflow =
        currentWorkflowFilter === "all" ||
        node.workflowId === currentWorkflowFilter;
      return matchesSearchTerm && matchesActionType && matchesWorkflow;
    });
  }, [
    allNodes,
    searchTerm,
    currentActionTypeFilter,
    currentWorkflowFilter,
    workflowVariables,
  ]);

  const handleNavigateToNode = useCallback(
    (node: SearchNodeResult) => {
      // Update selected node state
      setSelectedNodeId(node.id);

      if (node.workflowId !== currentWorkflowId) {
        onWorkflowOpen(node.workflowId);
      }
      setTimeout(
        () => {
          const reactFlowNode = getNode(node.id);
          if (reactFlowNode) {
            setCenter(
              reactFlowNode.position.x + (reactFlowNode.width ?? 0) / 2,
              reactFlowNode.position.y + (reactFlowNode.height ?? 0) / 2,
              { zoom: 1.1, duration: 300 },
            );
            if (
              prevSelectedNodeIdRef.current &&
              prevSelectedNodeIdRef.current !== node.id
            ) {
              onNodesChange?.([
                {
                  type: "select",
                  id: prevSelectedNodeIdRef.current,
                  selected: false,
                },
              ]);
            }
            onNodesChange?.([
              {
                type: "select",
                id: reactFlowNode.id,
                selected: true,
              },
            ]);
            prevSelectedNodeIdRef.current = node.id;
          }
        },
        node.workflowId !== currentWorkflowId ? 100 : 0,
      );
    },
    [currentWorkflowId, onWorkflowOpen, getNode, setCenter, onNodesChange],
  );

  const handleSingleClick = useCallback(
    (node?: SearchNodeResult) => {
      if (!node) return;
      setSelectedNodeId(node.id);
      if (
        prevSelectedNodeIdRef.current &&
        prevSelectedNodeIdRef.current !== node.id
      ) {
        onNodesChange?.([
          {
            type: "select",
            id: prevSelectedNodeIdRef.current,
            selected: false,
          },
        ]);
      }
      onNodesChange?.([
        {
          type: "select",
          id: node.id,
          selected: true,
        },
      ]);
      prevSelectedNodeIdRef.current = node.id;
    },
    [onNodesChange],
  );

  const handleDoubleClick = useCallback(
    (node?: SearchNodeResult) => {
      if (!node) return;
      handleNavigateToNode(node);
    },
    [handleNavigateToNode],
  );

  const [handleRowClick, handleRowDoubleClick] = useDoubleClick(
    handleSingleClick,
    handleDoubleClick,
    50,
  );

  return {
    filteredNodes,
    selectedNodeId,
    searchTerm,
    currentActionTypeFilter,
    currentWorkflowFilter,
    actionTypes,
    workflows,
    setSearchTerm,
    setCurrentActionTypeFilter,
    setCurrentWorkflowFilter,
    handleRowClick,
    handleRowDoubleClick,
  };
};
