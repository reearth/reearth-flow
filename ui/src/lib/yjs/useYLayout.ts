import { useCallback } from "react";
import * as Y from "yjs";

import { Direction, Edge, Node, Workflow } from "@flow/types";
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
    async (direction: Direction, xSpacing: number, ySpacing: number) => {
      // Phase 1: compute all layouts asynchronously before touching Yjs
      const layouts = await Promise.all(
        rawWorkflows.map(async (rawWorkflow) => {
          const nodes = rawWorkflow.nodes as Node[];
          const edges = rawWorkflow.edges as Edge[];
          const result = await autoLayout(
            direction,
            nodes,
            edges,
            xSpacing,
            ySpacing,
          );
          return { id: rawWorkflow.id, result };
        }),
      );

      // Phase 2: apply all results in one synchronous undo transaction
      undoTrackerActionWrapper(() => {
        layouts.forEach(({ id, result }) => {
          const yNodes = yWorkflows?.get(id)?.get("nodes") as
            | YNodesMap
            | undefined;
          if (!yNodes) return;
          result.nodes.forEach((n) => {
            yNodes.set(n.id, yNodeConstructor(n));
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
