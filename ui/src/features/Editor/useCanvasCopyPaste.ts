import {
  addEdge,
  EdgeChange,
  getNodesBounds,
  useReactFlow,
  useViewport,
  XYPosition,
} from "@xyflow/react";
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
  handleEdgesChange,
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
  handleEdgesChange: (changes: EdgeChange[]) => void;
}) => {
  const { copy, paste } = useCopyPaste();
  const { toast } = useToast();
  const t = useT();
  const { x, y, zoom } = useViewport();
  const { screenToFlowPosition } = useReactFlow();

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

  const calculateOffset = useCallback(
    (
      topLevelNodes: Node[],
      mousePosition?: XYPosition,
      isCutByShortCut?: boolean,
    ) => {
      let offsetX = 0;
      let offsetY = 0;
      const bounds = getNodesBounds(topLevelNodes);
      if (mousePosition) {
        const reactFlowPosition = {
          x: (mousePosition.x - x) / zoom,
          y: (mousePosition.y - y) / zoom,
        };

        offsetX = reactFlowPosition.x - bounds.x;
        offsetY = reactFlowPosition.y - bounds.y;
      } else if (isCutByShortCut && !mousePosition) {
        const viewportCenter = screenToFlowPosition({
          x: window.innerWidth / 2,
          y: window.innerHeight / 2,
        });

        const nodesCenterX = bounds.x + bounds.width / 2;
        const nodesCenterY = bounds.y + bounds.height / 2;

        offsetX = viewportCenter.x - nodesCenterX;
        offsetY = viewportCenter.y - nodesCenterY;
      } else {
        // if NOT a child of a batch, offset position for user's benefit
        offsetX = 25;
        offsetY = 25;
      }

      return { offsetX, offsetY };
    },
    [screenToFlowPosition, x, y, zoom],
  );

  const newNodeCreation = useCallback(
    (
      pastedNodes: Node[],
      mousePosition?: XYPosition,
      isCutByShortCut?: boolean,
    ): Node[] => {
      const newNodes: Node[] = [];
      const parentIdMapArray: { prevId: string; newId: string }[] = [];

      const nodesToCalculateOffset = pastedNodes.filter(
        (node) => !node.parentId,
      );

      const positionOffset = calculateOffset(
        nodesToCalculateOffset,
        mousePosition,
        isCutByShortCut,
      );

      for (const n of pastedNodes) {
        const newId = generateUUID();
        const newPosition = n.parentId
          ? { x: n.position.x, y: n.position.y }
          : {
              x: n.position.x + positionOffset.offsetX,
              y: n.position.y + positionOffset.offsetY,
            };

        const newNode = {
          ...n,
          id: newId,
          position: newPosition,
          selected: true,
          data: { ...n.data },
        };
        if (n.type === "batch") {
          parentIdMapArray.push({ prevId: n.id, newId });
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
    [calculateOffset],
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

  const prepareCopyData = useCallback(
    async (node?: Node) => {
      const selected: { nodes: Node[]; edges: Edge[] } | undefined = {
        nodes: node ? [node] : nodes.filter((n) => n.selected),
        edges: edges.filter((e) => e.selected),
      };
      let referencedWorkflows: Workflow[] = [];

      if (selected.nodes.length === 0 && selected.edges.length === 0) return;

      const processedNodeIds = new Set();
      const nodesToProcess = [...selected.nodes];
      const edgesToProcess = [...selected.edges];

      selected.nodes.forEach((node) => {
        processedNodeIds.add(node.id);
      });

      const batchNodeIds = selected.nodes
        .filter((node) => node.type === "batch")
        .map((node) => node.id);

      nodes.forEach((node) => {
        if (
          node.parentId &&
          batchNodeIds.includes(node.parentId) &&
          !processedNodeIds.has(node.id)
        ) {
          nodesToProcess.push(node);
          processedNodeIds.add(node.id);
        }
      });

      const processedEdgeIds = new Set(selected.edges.map((edge) => edge.id));

      edges.forEach((edge) => {
        if (
          (processedNodeIds.has(edge.source) ||
            processedNodeIds.has(edge.target)) &&
          !processedEdgeIds.has(edge.id)
        ) {
          edgesToProcess.push(edge);
          processedEdgeIds.add(edge.id);
        }
      });

      if (nodesToProcess.some((n) => n.type === "subworkflow")) {
        referencedWorkflows = collectSubworkflows(nodesToProcess, rawWorkflows);
        if (referencedWorkflows.length === 0) return;
      }

      console.log("TEST", nodesToProcess);
      return {
        nodes: nodesToProcess,
        edges: edgesToProcess,
        workflows: referencedWorkflows,
        copiedAt: Date.now(),
      };
    },
    [nodes, edges, collectSubworkflows, rawWorkflows],
  );

  const handleCopy = useCallback(
    async (node?: Node) => {
      const copyData = await prepareCopyData(node);
      if (!copyData) return;

      if (copyData.nodes.some((n) => n.type === "reader")) {
        return toast({
          title: t("Reader node cannot be copied"),
          description: t("Only one reader can be present in any project."),
          variant: "default",
        });
      }

      await copy({
        ...copyData,
      });
    },
    [copy, prepareCopyData, toast, t],
  );

  const handleCut = useCallback(
    async (isCutByShortCut?: boolean, node?: Node) => {
      const cutData = await prepareCopyData(node);
      if (!cutData) return;

      await copy({
        ...cutData,
        isCutByShortCut,
      });

      const nodeChanges: NodeChange[] = cutData.nodes.map((n) => ({
        id: n.id,
        type: "remove",
      }));

      const edgeChanges: EdgeChange[] = cutData.edges.map((e) => ({
        id: e.id,
        type: "remove",
      }));

      handleNodesChange(nodeChanges);
      handleEdgesChange(edgeChanges);
    },
    [prepareCopyData, handleNodesChange, handleEdgesChange, copy],
  );
  const handlePaste = useCallback(
    async (mousePosition?: XYPosition) => {
      const {
        nodes: pastedNodes,
        edges: pastedEdges,
        workflows: pastedWorkflows,
        isCutByShortCut,
      } = (await paste()) || {
        nodes: [],
        edges: [],
      };
      // Check for cut to ensure only one reader can be pasted
      if (
        pastedNodes.some((p: Node) => p.type === "reader") &&
        nodes.some((n) => n.type === "reader")
      ) {
        return toast({
          title: t("Reader node cannot be pasted"),
          description: t("Only one reader can be present in any project."),
          variant: "default",
        });
      }

      const newNodes = newNodeCreation(
        pastedNodes,
        mousePosition,
        isCutByShortCut,
      );
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
      t,
      toast,
    ],
  );

  return {
    handleCopy,
    handleCut,
    handlePaste,
  };
};
