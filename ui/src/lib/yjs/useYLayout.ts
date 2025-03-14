import { useCallback } from "react";
import * as Y from "yjs";

import { Algorithm, Direction, Edge, Node, Workflow } from "@flow/types";
import { autoLayout } from "@flow/utils/autoLayout";

import { yNodeConstructor } from "./conversions";
import { YNodesArray, YWorkflow } from "./types";

export default ({
  yWorkflows,
  rawWorkflows,
  undoTrackerActionWrapper,
}: {
  yWorkflows?: Y.Map<YWorkflow>;
  rawWorkflows: Workflow[];
  undoTrackerActionWrapper: (callback: () => void) => void;
}) => {
  const handleYLayoutChange = useCallback(
    (algorithm: Algorithm, direction: Direction, _spacing: number) => {
      undoTrackerActionWrapper(() => {
        const updatedRawWorkflows: Workflow[] = rawWorkflows.map(
          (rawWorkflow) => {
            const nodes = rawWorkflow.nodes as Node[];
            const edges = rawWorkflow.edges as Edge[];
            const layoutedElements = autoLayout(
              algorithm,
              direction,
              nodes,
              edges,
              // spacing,
            );

            return {
              ...rawWorkflow,
              nodes: layoutedElements.nodes,
              edges: layoutedElements.edges,
            };
          },
        );

        updatedRawWorkflows.forEach((rawWorkflow) => {
          const yWorkflow = yWorkflows?.get(rawWorkflow.id);
          const yNodes = yWorkflow?.get("nodes") as YNodesArray;

          if (!yWorkflow) {
            return;
          }
          const newYNodes = (rawWorkflow.nodes as Node[]).map((newNode) =>
            yNodeConstructor(newNode),
          );
          yNodes.delete(0, yNodes.length);
          yNodes.insert(0, newYNodes);
        });
      });
    },
    [rawWorkflows, yWorkflows, undoTrackerActionWrapper],
  );

  return {
    handleYLayoutChange,
  };
};
