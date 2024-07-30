import { useCallback, useMemo, useState } from "react";
import * as Y from "yjs";

import type { Edge, Node } from "@flow/types";
import { randomID } from "@flow/utils";

import { fromYjsText } from "./conversions";
import { useY } from "./useY";
import { yWorkflowBuilder, type YWorkflow, YNodesArray, YEdgesArray } from "./workflowBuilder";

export default ({
  workflowId,
  handleWorkflowIdChange,
}: {
  workflowId?: string;
  handleWorkflowIdChange: (id?: string) => void;
}) => {
  const [{ yWorkflows }] = useState(() => {
    // TODO: setup middleware/websocket provider
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
    const yWorkflow = yWorkflowBuilder("main", "Main Workflow");
    yWorkflows.push([yWorkflow]);

    return { yWorkflows };
  });

  const rawWorkflows = useY(yWorkflows);

  const [workflows, setWorkflows] = useState<{ id: string; name: string }[]>(
    rawWorkflows.map(w2 => ({
      id: fromYjsText(w2?.id as Y.Text),
      name: fromYjsText(w2?.name as Y.Text),
    })),
  );

  const [openWorkflowIds, setOpenWorkflowIds] = useState<string[]>(
    workflows.filter(w => w.id === "main").map(w => w.id),
  );

  const openWorkflows: {
    id: string;
    name: string;
  }[] = useMemo(
    () => workflows.filter(w => openWorkflowIds.includes(w.id)),
    [workflows, openWorkflowIds],
  );

  const handleWorkflowOpen = useCallback(
    (workflowId: string) => {
      setOpenWorkflowIds(ids => {
        handleWorkflowIdChange(workflowId);
        if (ids.includes(workflowId)) return ids;
        return [...ids, workflowId];
      });
    },
    [handleWorkflowIdChange],
  );

  const handleWorkflowClose = useCallback(
    (workflowId: string) => {
      setOpenWorkflowIds(ids => ids.filter(id => id !== workflowId));
      if (workflowId === "main") {
        handleWorkflowIdChange("main");
      }
    },
    [handleWorkflowIdChange],
  );

  const currentWorkflowIndex = useMemo(
    () => workflows.findIndex(w => w.id === workflowId),
    [workflowId, workflows],
  );

  const yWorkflow = yWorkflows.get(currentWorkflowIndex);

  const nodes = useY(yWorkflow?.get("nodes") ?? new Y.Array<Node>()) as Node[];
  const edges = useY(yWorkflow?.get("edges") ?? new Y.Array<Edge>()) as Edge[];

  const handleWorkflowAdd = useCallback(() => {
    // New workflow
    const workflowId = yWorkflows.length.toString() + "-workflow";
    const workflowName = "Sub Workflow-" + yWorkflows.length.toString();
    const newEntranceNode: Node = {
      id: randomID(),
      type: "entrance",
      position: { x: 200, y: 200 },
      data: {
        name: `New Entrance node`,
        outputs: ["target"],
        status: "idle",
        // locked: false,
        // onLock: onNodeLocking,
      },
    };
    const newExitNode: Node = {
      id: randomID(),
      type: "exit",
      position: { x: 1000, y: 200 },
      data: {
        name: `New Exit node`,
        inputs: ["source"],
        status: "idle",
        // locked: false,
        // onLock: onNodeLocking,
      },
    };

    const yWorkflow = yWorkflowBuilder(workflowId, workflowName, [newEntranceNode, newExitNode]);

    // Update main workflow
    const newSubworkflowNode: Node = {
      id: workflowId,
      type: "subworkflow",
      position: { x: 600, y: 200 },
      data: {
        name: workflowName,
        status: "idle",
        inputs: ["source"],
        outputs: ["target"],
        onDoubleClick: handleWorkflowOpen,
      },
    };
    const mainWorkflow = yWorkflows.get(0);

    const mainWorkflowNodes = mainWorkflow?.get("nodes") as YNodesArray | undefined;
    mainWorkflowNodes?.push([newSubworkflowNode]);

    yWorkflows.push([yWorkflow]);
    setWorkflows(w => [...w, { id: workflowId, name: workflowName }]);
    setOpenWorkflowIds(ids => [...ids, workflowId]);
  }, [yWorkflows, handleWorkflowOpen]);

  const handleWorkflowRemove = useCallback(
    (workflowId: string) => {
      const index = workflows.findIndex(w => w.id === workflowId);
      setWorkflows(w => w.filter(w => w.id !== workflowId));

      if (index === currentWorkflowIndex) {
        handleWorkflowIdChange("main");
      }
      yWorkflows.delete(index);

      // Remove subworkflow node from main workflow
      const mainWorkflow = yWorkflows.get(0);
      const mainWorkflowNodes = mainWorkflow?.get("nodes") as YNodesArray | undefined;
      const subworkflowIndex = mainWorkflowNodes
        ?.toJSON()
        .findIndex((n: Node) => n.id === workflowId);
      if (subworkflowIndex !== undefined && subworkflowIndex !== -1) {
        mainWorkflowNodes?.delete(subworkflowIndex);
      }
    },
    [workflows, yWorkflows, currentWorkflowIndex, handleWorkflowIdChange],
  );

  const handleNodesUpdate = (newNodes: Node[]) => {
    const yNodes = yWorkflow?.get("nodes") as YNodesArray | undefined;
    yNodes?.delete(0, nodes.length);
    yNodes?.insert(0, newNodes);
  };

  const handleEdgesUpdate = (newEdges: Edge[]) => {
    const yEdges = yWorkflow?.get("edges") as YEdgesArray | undefined;
    yEdges?.delete(0, edges.length);
    yEdges?.insert(0, newEdges);
  };

  return {
    nodes,
    edges,
    openWorkflows,
    handleWorkflowClose,
    handleWorkflowAdd,
    handleWorkflowRemove,
    handleNodesUpdate,
    handleEdgesUpdate,
  };
};
