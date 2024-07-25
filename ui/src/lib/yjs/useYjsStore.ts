import { useCallback, useMemo, useState } from "react";
import { useY } from "react-yjs";
import * as Y from "yjs";

import type { Edge, Node } from "@flow/types";

import { fromYjsText } from "./conversions";
import { yWorkflowBuilder } from "./workflowBuilder";

type YWorkflow = Y.Map<Y.Text | YNodesArray | YEdgesArray> | undefined;

type YNodesArray = Y.Array<Node>;

type YEdgesArray = Y.Array<Edge>;

export default ({
  workflowId,
  handleWorkflowIdChange,
}: {
  workflowId?: string;
  handleWorkflowIdChange: (id?: string) => void;
}) => {
  const [{ yWorkflows }] = useState(() => {
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
    const { yWorkflow } = yWorkflowBuilder("main", "Main Workflow");
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

  const currentWorkflowIndex = useMemo(
    () => workflows.findIndex(w => w.id === workflowId),
    [workflowId, workflows],
  );

  const yWorkflow = yWorkflows.get(currentWorkflowIndex);

  const nodes = useY(yWorkflow?.get("nodes") ?? new Y.Array<Node>()) as Node[];
  const edges = useY(yWorkflow?.get("edges") ?? new Y.Array<Edge>()) as Edge[];

  const handleWorkflowAdd = useCallback(() => {
    const workflowId = yWorkflows.length.toString() + "-workflow";
    const workflowName = "Sub Workflow-" + yWorkflows.length.toString();

    const yWorkflow = yWorkflowBuilder(workflowId, workflowName).yWorkflow;
    yWorkflows.push([yWorkflow]);

    setWorkflows(w => [...w, { id: workflowId, name: workflowName }]);
    handleWorkflowIdChange(workflowId);
  }, [yWorkflows, handleWorkflowIdChange]);

  const handleWorkflowRemove = useCallback(
    (workflowId: string) => {
      const index = workflows.findIndex(w => w.id === workflowId);
      setWorkflows(w => w.filter(w => w.id !== workflowId));

      if (index === currentWorkflowIndex) {
        handleWorkflowIdChange("main");
      }
      yWorkflows.delete(index);
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
    workflows,
    handleWorkflowAdd,
    handleWorkflowRemove,
    handleNodesUpdate,
    handleEdgesUpdate,
  };
};
