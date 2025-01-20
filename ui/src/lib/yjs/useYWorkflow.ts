import { XYPosition } from "@xyflow/react";
import { Dispatch, SetStateAction, useCallback } from "react";
import { Array as YArray } from "yjs";

import { config } from "@flow/config";
import {
  DEFAULT_ENTRY_GRAPH_ID,
  DEFAULT_ROUTING_PORT,
} from "@flow/global-constants";
import type { Action, Edge, Node } from "@flow/types";
import { generateUUID } from "@flow/utils";

import { fetcher } from "../fetch/transformers/useFetch";

import { YNodesArray, YWorkflow, yWorkflowBuilder } from "./utils";

export default ({
  yWorkflows,
  rawWorkflows,
  currentWorkflowId,
  undoTrackerActionWrapper,
  setWorkflows,
  setOpenWorkflowIds,
}: {
  yWorkflows: YArray<YWorkflow>;
  rawWorkflows: Record<string, string | Node[] | Edge[]>[];
  currentWorkflowId: string;
  undoTrackerActionWrapper: (callback: () => void) => void;
  setWorkflows: Dispatch<
    SetStateAction<
      {
        id: string;
        name: string;
      }[]
    >
  >;
  setOpenWorkflowIds: Dispatch<SetStateAction<string[]>>;
}) => {
  const { api } = config();
  const currentYWorkflow = yWorkflows.get(
    rawWorkflows.findIndex((w) => w.id === currentWorkflowId) || 0,
  );

  const createWorkflow = useCallback(
    async (
      workflowId: string,
      workflowName: string,
      position: XYPosition,
      initialNodes?: Node[],
      initialEdges?: Edge[],
    ) => {
      const [inputRouter, outputRouter] = await Promise.all([
        fetcher<Action>(`${api}/actions/InputRouter`),
        fetcher<Action>(`${api}/actions/OutputRouter`),
      ]);

      const inputNodeId = generateUUID();
      const newInputNode: Node = {
        id: inputNodeId,
        type: inputRouter.type,
        position: { x: 200, y: 200 },
        data: {
          officialName: inputRouter.name,
          outputs: inputRouter.outputPorts,
          status: "idle",
          params: {
            routingPort: DEFAULT_ROUTING_PORT,
          },
        },
      };

      const outputNodeId = generateUUID();
      const newOutputNode: Node = {
        id: outputNodeId,
        type: outputRouter.type,
        position: { x: 1000, y: 200 },
        data: {
          officialName: outputRouter.name,
          inputs: outputRouter.inputPorts,
          status: "idle",
          params: {
            routingPort: DEFAULT_ROUTING_PORT,
          },
        },
      };

      const workflowNodes = [
        newInputNode,
        ...(initialNodes ?? []),
        newOutputNode,
      ];
      const newYWorkflow = yWorkflowBuilder(
        workflowId,
        workflowName,
        workflowNodes,
        initialEdges,
      );

      const newSubworkflowNode: Node = {
        id: workflowId,
        type: "subworkflow",
        position,
        data: {
          officialName: workflowName,
          status: "idle",
          pseudoInputs: [
            { nodeId: inputNodeId, portName: DEFAULT_ROUTING_PORT },
          ],
          pseudoOutputs: [
            { nodeId: outputNodeId, portName: DEFAULT_ROUTING_PORT },
          ],
        },
        selected: true,
      };

      return { newYWorkflow, newSubworkflowNode };
    },
    [api],
  );

  const handleWorkflowAdd = useCallback(
    (position: XYPosition = { x: 600, y: 200 }) =>
      undoTrackerActionWrapper(async () => {
        const workflowId = generateUUID();
        const workflowName = `Sub Workflow-${yWorkflows.length}`;

        const { newYWorkflow, newSubworkflowNode } = await createWorkflow(
          workflowId,
          workflowName,
          position,
        );

        const parentWorkflow = yWorkflows.get(
          rawWorkflows.findIndex((w) => w.id === currentWorkflowId) || 0,
        );
        const parentWorkflowNodes = parentWorkflow?.get("nodes") as
          | YNodesArray
          | undefined;
        parentWorkflowNodes?.push([newSubworkflowNode]);

        yWorkflows.push([newYWorkflow]);
        setWorkflows((w) => [...w, { id: workflowId, name: workflowName }]);
        setOpenWorkflowIds((ids) => [...ids, workflowId]);
      }),
    [
      yWorkflows,
      currentWorkflowId,
      rawWorkflows,
      createWorkflow,
      undoTrackerActionWrapper,
      setWorkflows,
      setOpenWorkflowIds,
    ],
  );

  const handleWorkflowAddFromSelection = useCallback(
    (nodes: Node[], edges: Edge[]) =>
      undoTrackerActionWrapper(async () => {
        const nodesByParentId = new Map<string, Node[]>();
        nodes.forEach((node) => {
          if (node.parentId) {
            if (!nodesByParentId.has(node.parentId)) {
              nodesByParentId.set(node.parentId, []);
            }
            nodesByParentId.get(node.parentId)?.push(node);
          }
        });

        const selectedNodes = nodes.filter((n) => n.selected);
        if (selectedNodes.length === 0) return;

        const getBatchNodes = (batchId: string): Node[] =>
          nodesByParentId.get(batchId) ?? [];

        const allIncludedNodeIds = new Set<string>();
        selectedNodes.forEach((node) => {
          allIncludedNodeIds.add(node.id);
          if (node.type === "batch") {
            getBatchNodes(node.id).forEach((batchNode) =>
              allIncludedNodeIds.add(batchNode.id),
            );
          }
        });

        const allIncludedNodes = nodes.filter((n) =>
          allIncludedNodeIds.has(n.id),
        );
        const position = {
          x: Math.min(...selectedNodes.map((n) => n.position.x)),
          y: Math.min(...selectedNodes.map((n) => n.position.y)),
        };

        const adjustedNodes = allIncludedNodes.map((node) => ({
          ...node,
          position: node.parentId
            ? node.position
            : {
                x: node.position.x - position.x + 400,
                y: node.position.y - position.y + 200,
              },
          selected: false,
        }));

        const internalEdges = edges.filter(
          (e) =>
            allIncludedNodeIds.has(e.source) &&
            allIncludedNodeIds.has(e.target),
        );

        const workflowId = generateUUID();
        const workflowName = `Sub Workflow-${yWorkflows.length}`;

        const { newYWorkflow, newSubworkflowNode } = await createWorkflow(
          workflowId,
          workflowName,
          position,
          adjustedNodes,
          internalEdges,
        );

        const parentWorkflow = yWorkflows.get(
          rawWorkflows.findIndex((w) => w.id === currentWorkflowId) || 0,
        );
        const parentWorkflowNodes = parentWorkflow?.get("nodes") as
          | YNodesArray
          | undefined;
        const remainingNodes = nodes.filter(
          (n) => !allIncludedNodeIds.has(n.id),
        );

        parentWorkflowNodes?.delete(0, parentWorkflowNodes.length);
        parentWorkflowNodes?.push([...remainingNodes, newSubworkflowNode]);

        yWorkflows.push([newYWorkflow]);
        setWorkflows((w) => [...w, { id: workflowId, name: workflowName }]);
        setOpenWorkflowIds((ids) => [...ids, workflowId]);
      }),
    [
      yWorkflows,
      currentWorkflowId,
      rawWorkflows,
      createWorkflow,
      undoTrackerActionWrapper,
      setWorkflows,
      setOpenWorkflowIds,
    ],
  );

  const handleWorkflowUpdate = useCallback(
    (workflowId: string, nodes?: Node[], edges?: Edge[]) => {
      const workflowName = "Sub Workflow-" + yWorkflows.length.toString();
      const newYWorkflow = yWorkflowBuilder(
        workflowId,
        workflowName,
        nodes,
        edges,
      );
      yWorkflows.push([newYWorkflow]);
      setWorkflows((w) => [...w, { id: workflowId, name: workflowName }]);
    },
    [setWorkflows, yWorkflows],
  );

  const handleWorkflowsRemove = useCallback(
    (nodeIds: string[]) =>
      undoTrackerActionWrapper(() => {
        const workflowIds: string[] = [];
        const localWorkflows = [...rawWorkflows];

        const removeNodes = (nodeIds: string[]) => {
          nodeIds.forEach((nid) => {
            if (nid === DEFAULT_ENTRY_GRAPH_ID) return;

            const index = localWorkflows.findIndex((w) => w.id === nid);
            if (index === -1) return;

            // Loop over workflow at current index and remove any subworkflow nodes
            (localWorkflows[index].nodes as Node[]).forEach((node) => {
              if (node.type === "subworkflow") {
                removeNodes([node.id]);
              }
            });

            workflowIds.push(nid);
            yWorkflows.delete(index);
            localWorkflows.splice(index, 1);
          });
        };

        removeNodes(nodeIds);

        setWorkflows((w) => w.filter((w) => !workflowIds.includes(w.id)));
        setOpenWorkflowIds((ids) =>
          ids.filter((id) => !workflowIds.includes(id)),
        );
      }),
    [
      rawWorkflows,
      yWorkflows,
      undoTrackerActionWrapper,
      setWorkflows,
      setOpenWorkflowIds,
    ],
  );

  const handleWorkflowRename = useCallback(
    (id: string, name: string) =>
      undoTrackerActionWrapper(() => {
        if (!name.trim()) {
          throw new Error("Workflow name cannot be empty");
        }

        // Update local state
        setWorkflows((w) => w.map((w) => (w.id === id ? { ...w, name } : w)));

        // Update subworkflow node in main workflow if this is a subworkflow
        const mainWorkflow = yWorkflows.get(0);
        const mainWorkflowNodes = mainWorkflow?.get("nodes") as YNodesArray;

        for (const node of mainWorkflowNodes) {
          if (node.id === id) {
            node.data = {
              ...node.data,
              customName: name,
            };
          }
        }
      }),
    [undoTrackerActionWrapper, yWorkflows, setWorkflows],
  );

  return {
    currentYWorkflow,
    handleWorkflowAdd,
    handleWorkflowUpdate,
    handleWorkflowsRemove,
    handleWorkflowRename,
    handleWorkflowAddFromSelection,
  };
};
