import { useCallback } from "react";
import * as Y from "yjs";

import { Algorithm, Direction, Edge, Node, Workflow } from "@flow/types";
import { autoLayout } from "@flow/utils/autoLayout";

import { updateYNodePosition } from "./conversions";
import { YNodesMap, YWorkflow } from "./types";

export default ({
  currentWorkflowId,
  yWorkflows,
  rawWorkflows,
  undoTrackerActionWrapper,
}: {
  currentWorkflowId: string;
  yWorkflows?: Y.Map<YWorkflow>;
  rawWorkflows: Workflow[];
  undoTrackerActionWrapper: (
    callback: () => void,
    originPrepend?: string,
  ) => void;
}) => {
  const handleYLayoutChange = useCallback(
    (algorithm: Algorithm, direction: Direction, applyToAll: boolean) => {
      const targets = applyToAll
        ? rawWorkflows
        : rawWorkflows.filter((w) => w.id === currentWorkflowId);
      undoTrackerActionWrapper(() => {
        targets.forEach((rawWorkflow) => {
          const yNodes = yWorkflows?.get(rawWorkflow.id)?.get("nodes") as
            | YNodesMap
            | undefined;
          if (!yNodes) return;

          const nodes = rawWorkflow.nodes as Node[];
          const edges = rawWorkflow.edges as Edge[];
          const layoutedElements = autoLayout(
            algorithm,
            direction,
            nodes,
            edges,
          );

          layoutedElements.nodes?.forEach((n) => {
            const yNode = yNodes.get(n.id);
            if (!yNode) return;
            updateYNodePosition(yNode, n.position);
          });
        });
      });
    },
    [currentWorkflowId, rawWorkflows, yWorkflows, undoTrackerActionWrapper],
  );

  return {
    handleYLayoutChange,
  };
};
