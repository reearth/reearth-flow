import { addEdge, useViewport, XYPosition } from "@xyflow/react";
import { useCallback } from "react";

import { useCopyPaste } from "@flow/hooks/useCopyPaste";
import { useT } from "@flow/lib/i18n";
import type { Edge, Node, NodeChange, Workflow } from "@flow/types";
import { generateUUID } from "@flow/utils";

import { useToast } from "../NotificationSystem/useToast";

export default ({
  nodes,
  edges,
  rawWorkflows,
  handleWorkflowUpdate,
  handleNodesAdd,
  handleNodesChange,
  handleEdgesAdd,
}: {
  nodes: Node[];
  edges: Edge[];
  rawWorkflows: Workflow[];
  handleWorkflowUpdate: (
    workflowId: string,
    nodes?: Node[],
    edges?: Edge[],
  ) => void;
  handleNodesAdd: (newNodes: Node[]) => void;
  handleNodesChange: (changes: NodeChange[]) => void;
  handleEdgesAdd: (newEdges: Edge[]) => void;
}) => {
  const { copy, paste } = useCopyPaste();
  const { toast } = useToast();
  const t = useT();
  const { x, y, zoom } = useViewport();

  const newEdgeCreation = useCallback(
    (pastedEdges: Edge[], oldNodes: Node[], newNodes: Node[]): Edge[] => {
      let newEdges: Edge[] = [];
      for (const e of pastedEdges) {
        const sourceNode =
          newNodes[oldNodes?.findIndex((n) => n.id === e.source)];
        const targetNode =
          newNodes[oldNodes?.findIndex((n) => n.id === e.target)];

        if (!sourceNode || !targetNode) continue;

        newEdges = addEdge(
          {
            id: generateUUID(),
            source: sourceNode.id,
            target: targetNode.id,
            sourceHandle: e.sourceHandle ?? null,
            targetHandle: e.targetHandle ?? null,
          },
          newEdges,
        );
      }
      return newEdges;
    },
    [],
  );

  const newNodeCreation = useCallback(
    (pastedNodes: Node[], mousePosition?: XYPosition): Node[] => {
      const newNodes: Node[] = [];
      const parentIdMapArray: { prevId: string; newId: string }[] = [];

      let offsetX = 0;
      let offsetY = 0;

      if (mousePosition) {
        const reactFlowPosition = {
          x: (mousePosition.x - x) / zoom,
          y: (mousePosition.y - y) / zoom,
        };

        let minX = Infinity;
        let minY = Infinity;

        for (const node of pastedNodes) {
          if (!node.parentId) {
            if (node.position.x < minX) minX = node.position.x;
            if (node.position.y < minY) minY = node.position.y;
          }
        }

        offsetX = reactFlowPosition.x - minX;
        offsetY = reactFlowPosition.y - minY;
      } else {
        offsetX = 25;
        offsetY = 25;
      }

      for (const n of pastedNodes) {
        // if NOT a child of a batch, offset position for user's benefit
        const newId = generateUUID();
        const newPosition = n.parentId
          ? { x: n.position.x, y: n.position.y }
          : { x: n.position.x + offsetX, y: n.position.y + offsetY };

        const newNode = {
          ...n,
          id: newId,
          position: newPosition,
          selected: true,
          data: { ...n.data },
        };
        if (n.type === "batch") {
          parentIdMapArray.push({ prevId: n.id, newId });

          nodes.forEach((child) => {
            if (child.parentId === n.id) {
              const childNewNode = {
                ...child,
                id: generateUUID(),
                parentId: newId,
              };

              newNodes.push(childNewNode);
            }
          });
        }

        newNodes.push(newNode);
      }

      // Update parentIds for nodes that are batched
      const reBatchedNodes: Node[] = newNodes.map((nn) => {
        const rbn = nn;
        const newParentId = parentIdMapArray.find(
          (idMap) => idMap.prevId === nn.parentId,
        )?.newId;
        if (newParentId) {
          return { ...rbn, parentId: newParentId };
        }
        return rbn;
      });

      return reBatchedNodes;
    },
    [nodes, x, y, zoom],
  );
  const newWorkflowCreation = useCallback(
    (nodes: Node[], pastedWorkflows: Workflow[]) => {
      const newWorkflows: Workflow[] = [];

      const processSubworkflow = (node: Node): Node => {
        const subworkflowId = generateUUID();
        const originalSubworkflow = pastedWorkflows.find(
          (w) => w.id === node.data.subworkflowId,
        );

        if (!originalSubworkflow) return node;

        const newSubWorkflowNode = {
          ...node,
          data: {
            ...node.data,
            subworkflowId,
          },
        };

        const updatedSubworkflowNodes = originalSubworkflow.nodes?.map(
          (subNode) =>
            subNode.type === "subworkflow"
              ? processSubworkflow(subNode)
              : subNode,
        );

        const newSubworkflow = {
          ...originalSubworkflow,
          id: subworkflowId,
          nodes: updatedSubworkflowNodes,
        };

        newWorkflows.push(newSubworkflow);
        return newSubWorkflowNode;
      };

      const processedNewNodes = nodes.map((n) =>
        n.type === "subworkflow" ? processSubworkflow(n) : n,
      );

      return { newWorkflows, processedNewNodes };
    },
    [],
  );

  const collectSubworkflows = useCallback(
    (
      nodesToCheck: Node[],
      workflows: Workflow[],
      referencedWorkflows = new Set<string>(),
    ): Workflow[] => {
      let collectedWorkflows: Workflow[] = [];

      for (const node of nodesToCheck) {
        if (node.type === "subworkflow" && node.data.subworkflowId) {
          const subworkflow = workflows.find(
            (w) => w.id === node.data.subworkflowId,
          );
          if (referencedWorkflows.has(node.data.subworkflowId)) continue;
          if (subworkflow) {
            referencedWorkflows.add(node.data.subworkflowId);
            collectedWorkflows.push(subworkflow);
            const subworkflowNodes = subworkflow.nodes as Node[];
            collectedWorkflows = collectedWorkflows.concat(
              collectSubworkflows(
                subworkflowNodes,
                workflows,
                referencedWorkflows,
              ),
            );
          }
        }
      }

      return collectedWorkflows;
    },
    [],
  );

  const handleCopy = useCallback(async () => {
    const selected: { nodes: Node[]; edges: Edge[] } | undefined = {
      nodes: nodes.filter((n) => n.selected),
      edges: edges.filter((e) => e.selected),
    };
    let referencedWorkflows: Workflow[] = [];
    if (selected.nodes.some((n) => n.type === "reader"))
      return toast({
        title: t("Reader node cannot be copied"),
        description: t("Only one reader can be present in any project."),
        variant: "default",
      });

    if (selected.nodes.length === 0 && selected.edges.length === 0) return;

    if (selected.nodes.some((n) => n.type === "subworkflow")) {
      referencedWorkflows = collectSubworkflows(selected.nodes, rawWorkflows);
      if (referencedWorkflows.length === 0) return;
    }

    await copy({
      nodes: selected.nodes,
      edges: selected.edges,
      workflows: referencedWorkflows,
      copiedAt: Date.now(),
    });
  }, [nodes, edges, collectSubworkflows, copy, rawWorkflows, toast, t]);

  const handlePaste = useCallback(
    async (mousePosition?: XYPosition) => {
      const {
        nodes: pastedNodes,
        edges: pastedEdges,
        workflows: pastedWorkflows,
      } = (await paste()) || {
        nodes: [],
        edges: [],
      };

      const newNodes = newNodeCreation(pastedNodes, mousePosition);
      const newEdges = newEdgeCreation(pastedEdges, pastedNodes, newNodes);
      const { newWorkflows, processedNewNodes } = newWorkflowCreation(
        newNodes,
        pastedWorkflows,
      );

      // deselect all previously selected nodes
      const nodeChanges: NodeChange[] = nodes.map((n) => ({
        id: n.id,
        type: "select",
        selected: false,
      }));

      handleNodesChange(nodeChanges);

      handleNodesAdd([...processedNewNodes]);

      handleEdgesAdd(newEdges);

      newWorkflows.forEach((w) => {
        handleWorkflowUpdate(w.id, w.nodes, w.edges);
      });

      copy({
        nodes: processedNewNodes,
        edges: newEdges,
        workflows: newWorkflows,
        copiedAt: Date.now(),
      });

      return pastedNodes;
    },
    [
      nodes,
      copy,
      paste,
      handleNodesAdd,
      handleNodesChange,
      handleEdgesAdd,
      newNodeCreation,
      newEdgeCreation,
      newWorkflowCreation,
      handleWorkflowUpdate,
    ],
  );

  return {
    handleCopy,
    handlePaste,
  };
};
