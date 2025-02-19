import { useCallback } from "react";
import * as Y from "yjs";

import { Algorithm, Direction, Edge, Node } from "@flow/types";
import { autoLayout } from "@flow/utils/autoLayout";

import { yNodeConstructor } from "./conversions";
import { YNodesArray, YWorkflow } from "./types";

export default ({
  yWorkflows,
  rawWorkflows,
  undoTrackerActionWrapper,
}: {
  yWorkflows: Y.Array<YWorkflow>;
  rawWorkflows: Record<string, string | Node[] | Edge[]>[];
  undoTrackerActionWrapper: (callback: () => void) => void;
}) => {
  const handleYLayoutChange = useCallback(
    (algorithm: Algorithm, direction: Direction, _spacing: number) => {
      undoTrackerActionWrapper(() => {
        const updatedRawWorkflows: Record<string, string | Node[] | Edge[]>[] =
          rawWorkflows.map((rawWorkflow) => {
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
          });

        updatedRawWorkflows.forEach((rawWorkflow, index) => {
          const yWorkflow = yWorkflows.get(index);
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
