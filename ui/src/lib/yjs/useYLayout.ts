import { useCallback } from "react";
import * as Y from "yjs";

import { Algorithm, Direction, Edge, Node, Workflow } from "@flow/types";
import { autoLayout } from "@flow/utils/autoLayout";

import { yNodeConstructor } from "./conversions";
import { YNodesMap, YWorkflow } from "./types";

export default ({
  yWorkflows,
  rawWorkflows,
  undoTrackerActionWrapper,
}: {
  yWorkflows?: Y.Map<YWorkflow>;
  rawWorkflows: Workflow[];
  undoTrackerActionWrapper: (
    callback: () => void,
    originPrepend?: string,
  ) => void;
}) => {
  const handleYLayoutChange = useCallback(
    (algorithm: Algorithm, direction: Direction, _spacing: number) => {
      undoTrackerActionWrapper(() => {
        rawWorkflows.forEach((rawWorkflow) => {
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
            // spacing,
          );

          layoutedElements.nodes?.forEach((n) => {
            const yNode = yNodeConstructor(n);
            yNodes.set(n.id, yNode);
          });
        });
      });
    },
    [rawWorkflows, yWorkflows, undoTrackerActionWrapper],
  );

  return {
    handleYLayoutChange,
  };
};
