import { addEdge, EdgeChange } from "@xyflow/react";
import { useCallback } from "react";

import { useCopyPaste } from "@flow/hooks/useCopyPaste";
import type { Edge, Node, NodeChange } from "@flow/types";
import { generateUUID } from "@flow/utils";

export default ({
  nodes,
  edges,
  rawWorkflows,
  handleWorkflowUpdate,
  handleNodesAdd,
  handleNodesChange,
  handleEdgesAdd,
  handleEdgesChange,
}: {
  nodes: Node[];
  edges: Edge[];
  rawWorkflows: Record<string, string | Node[] | Edge[]>[];
  handleWorkflowUpdate: (
    workflowId: string,
    nodes?: Node[],
    edges?: Edge[],
  ) => void;
  handleNodesAdd: (newNodes: Node[]) => void;
  handleNodesChange: (changes: NodeChange[]) => void;
  handleEdgesAdd: (newEdges: Edge[]) => void;
  handleEdgesChange: (changes: EdgeChange[]) => void;
}) => {
  const { copy, cut, paste } = useCopyPaste();

  const getRelatedNodes = useCallback(
    (selectedNodeIds: string[]) => {
      const allRelatedNodes: Node[] = [];
      const processedIds = new Set<string>();

      const processNode = (nodeId: string) => {
        if (processedIds.has(nodeId)) return;
        processedIds.add(nodeId);

        const node = nodes.find((n) => n.id === nodeId);
        if (!node) return;
        allRelatedNodes.push(node);

        if (node.type === "batch") {
          const children = nodes.filter((n) => n.parentId === node.id);
          children.forEach((child) => processNode(child.id));
        }
      };
      selectedNodeIds.forEach((nodeId) => {
        processNode(nodeId);
      });
      return allRelatedNodes;
    },
    [nodes],
  );

  const handleCopy = useCallback(async () => {
    const selected: { nodeIds: string[]; edges: Edge[] } | undefined = {
      nodeIds: nodes.filter((n) => n.selected).map((n) => n.id),
      edges: edges.filter((e) => e.selected),
    };

    const selectedNodes = nodes.filter((n) => selected.nodeIds.includes(n.id));
    if (selectedNodes.some((n) => n.type === "reader")) return;

    if (selected.nodeIds.length === 0 && selected.edges.length === 0) return;
    await copy(selected);
  }, [nodes, edges, copy]);

  const handleCut = useCallback(async () => {
    const selected: { nodeIds: string[]; edges: Edge[] } | undefined = {
      nodeIds: nodes.filter((n) => n.selected).map((n) => n.id),
      edges: edges.filter((e) => e.selected),
    };

    const selectedNodes = nodes.filter((n) => selected?.nodeIds.includes(n.id));
    if (selectedNodes.some((n) => n.type === "reader")) return;

    if (selected.nodeIds.length === 0 && selected.edges.length === 0) return;

    const allNodes = getRelatedNodes(selected.nodeIds);
    const allNodeIds = allNodes.map((n) => n.id);

    const allEdges = edges.filter(
      (edge) =>
        allNodeIds.includes(edge.source) || allNodeIds.includes(edge.target),
    );

    await cut({
      nodeIds: selected.nodeIds,
      edges: allEdges,
      nodes: allNodes,
    });

    handleNodesChange(allNodeIds.map((id) => ({ id, type: "remove" })));

    handleEdgesChange(allEdges.map((e) => ({ id: e.id, type: "remove" })));
  }, [
    nodes,
    edges,
    cut,
    getRelatedNodes,
    handleNodesChange,
    handleEdgesChange,
  ]);

  const handlePaste = useCallback(async () => {
    const {
      nodeIds: pnid,
      edges: pastedEdges,
      nodes: pastedCutNodes,
    } = (await paste()) || {
      nodeIds: [],
      edges: [],
    };

    const copiedPastedNodes = nodes.filter((n) => pnid.includes(n.id));
    const pastedNodes = pastedCutNodes ? pastedCutNodes : copiedPastedNodes;

    const newEdgeCreation = (
      pe: Edge[],
      oldNodes: Node[],
      newNodes: Node[],
    ): Edge[] => {
      let newEdges: Edge[] = [];
      for (const e of pe) {
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
    };

    const newNodeCreation = (pn: Node[]): Node[] => {
      const newNodes: Node[] = [];

      const parentIdMapArray: { prevId: string; newId: string }[] = [];

      for (const n of pn) {
        // if NOT a child of a batch, offset position for user's benefit
        const newPosition = n.parentId
          ? { x: n.position.x, y: n.position.y }
          : { x: n.position.x + 40, y: n.position.y + 20 };
        const newNode: Node = {
          ...n,
          id: generateUUID(),
          position: newPosition,
          selected: true, // select pasted nodes
          data: {
            ...n.data,
          },
        };

        if (newNode.type === "batch") {
          parentIdMapArray.push({ prevId: n.id, newId: newNode.id });
        } else if (newNode.type === "subworkflow") {
          const subworkflowId = generateUUID();
          const subworkflowNodes = (rawWorkflows.find(
            (w) => w.id === n.data.subworkflowId,
          )?.nodes ?? []) as Node[];

          const newSubworkflowNodes = newNodeCreation(subworkflowNodes);

          const oldEdges = (rawWorkflows.find(
            (w) => w.id === n.data.subworkflowId,
          )?.edges ?? []) as Edge[];

          const newSubworkflowEdges = newEdgeCreation(
            oldEdges,
            subworkflowNodes,
            newSubworkflowNodes,
          );

          newNode.data.subworkflowId = subworkflowId;

          handleWorkflowUpdate(
            subworkflowId,
            newSubworkflowNodes,
            newSubworkflowEdges,
          );
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
    };

    const newNodes = newNodeCreation(pastedNodes);
    const newEdges = newEdgeCreation(pastedEdges, pastedNodes, newNodes);

    // Copy new nodes and edges. Since they are selected now,
    // if the user pastes again, the new nodes and edges will
    // be what is pasted with an appropriate offset position.
    copy({
      nodeIds: newNodes.map((n) => n.id),
      edges: newEdges,
    });

    cut({
      nodeIds: newNodes.map((n) => n.id),
      edges: newEdges,
    });

    // deselect all previously selected nodes
    const nodeChanges: NodeChange[] = nodes.map((n) => ({
      id: n.id,
      type: "select",
      selected: false,
    }));

    handleNodesChange(nodeChanges);

    handleNodesAdd([...newNodes]);

    handleEdgesAdd(newEdges);
  }, [
    nodes,
    rawWorkflows,
    copy,
    cut,
    paste,
    handleWorkflowUpdate,
    handleNodesAdd,
    handleNodesChange,
    handleEdgesAdd,
  ]);

  return {
    handleCopy,
    handleCut,
    handlePaste,
  };
};
