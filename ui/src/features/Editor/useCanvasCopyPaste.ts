import { addEdge } from "@xyflow/react";
import { useCallback, useState } from "react";

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
  const [hasItemsToPaste, setHasItemsToPaste] = useState<boolean>(false);

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
    (pastedNodes: Node[]): Node[] => {
      const newNodes: Node[] = [];
      const parentIdMapArray: { prevId: string; newId: string }[] = [];
      for (const n of pastedNodes) {
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

          nodes.forEach((child) => {
            if (child.parentId === n.id) {
              const childNewNode = {
                ...child,
                id: generateUUID(),
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
    [nodes],
  );

  const newWorkflowCreation = useCallback(
    (nodes: Node[], pastedWorkflows: Workflow[]) => {
      const newWorkflows: Workflow[] = [];

      const processSubworkflow = (node: Node) => {
        const subworkflowId = generateUUID();
        const originalSubworkflow = pastedWorkflows.find(
          (w) => w.id === node.data.subworkflowId,
        );

        if (!originalSubworkflow) return;

        const newSubworkflow = {
          ...originalSubworkflow,
          id: subworkflowId,
        };

        const newSubworkflowNodes = newNodeCreation(newSubworkflow.nodes ?? []);

        const newSubworkflowEdges = newEdgeCreation(
          newSubworkflow.edges ?? [],
          newSubworkflow.nodes ?? [],
          newSubworkflowNodes,
        );

        node.data.subworkflowId = subworkflowId;

        handleWorkflowUpdate(
          subworkflowId,
          newSubworkflowNodes,
          newSubworkflowEdges,
        );

        newSubworkflowNodes.forEach((subNode) => {
          if (subNode.type === "subworkflow") {
            processSubworkflow(subNode);
          }
        });

        newWorkflows.push(newSubworkflow);
      };

      for (const n of nodes) {
        if (n.type === "subworkflow") {
          processSubworkflow(n);
        }
      }

      return newWorkflows;
    },
    [newEdgeCreation, newNodeCreation, handleWorkflowUpdate],
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
    let newWorkflows: Workflow[] = [];
    if (selected.nodes.some((n) => n.type === "reader"))
      return toast({
        title: t("Reader node cannot be copied"),
        description: t("Only one reader can be present in any project."),
        variant: "default",
      });

    if (selected.nodes.length === 0 && selected.edges.length === 0) return;

    if (selected.nodes.some((n) => n.type === "subworkflow")) {
      newWorkflows = collectSubworkflows(selected.nodes, rawWorkflows);
      if (newWorkflows.length === 0) return;
    }

    setHasItemsToPaste(true);

    await copy({
      nodes: selected.nodes,
      edges: selected.edges,
      workflows: newWorkflows,
    });
  }, [nodes, edges, collectSubworkflows, copy, rawWorkflows, toast, t]);

  const handlePaste = useCallback(async () => {
    const {
      nodes: pastedNodes,
      edges: pastedEdges,
      workflows: pastedWorkflows,
    } = (await paste()) || {
      nodes: [],
      edges: [],
    };

    const newNodes = newNodeCreation(pastedNodes);
    const newEdges = newEdgeCreation(pastedEdges, pastedNodes, newNodes);
    const newWorkflows = newWorkflowCreation(newNodes, pastedWorkflows);
    // Copy new nodes and edges. Since they are selected now,
    // if the user pastes again, the new nodes and edges will
    // be what is pasted with an appropriate offset position.

    copy({
      nodes: newNodes,
      edges: newEdges,
      workflows: newWorkflows,
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

    return pastedNodes;
  }, [
    nodes,
    copy,
    paste,
    handleNodesAdd,
    handleNodesChange,
    handleEdgesAdd,
    newNodeCreation,
    newEdgeCreation,
    newWorkflowCreation,
  ]);

  return {
    handleCopy,
    handlePaste,
    hasItemsToPaste,
  };
};
