import { useCallback } from "react";
import * as Y from "yjs";

import { DEFAULT_GRID_SIZE } from "@flow/global-constants";
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
    (
      algorithm: Algorithm,
      direction: Direction,
      xSpacing: number,
      ySpacing: number,
    ) => {
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
            xSpacing,
            ySpacing,
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

  // Scales existing node positions relative to their bounding box centre.
  // Does not re-run any layout algorithm — preserves the user's arrangement.
  const handleYSpacingChange = useCallback(
    (xScale: number, yScale: number) => {
      undoTrackerActionWrapper(() => {
        rawWorkflows.forEach((rawWorkflow) => {
          const yNodes = yWorkflows?.get(rawWorkflow.id)?.get("nodes") as
            | YNodesMap
            | undefined;
          if (!yNodes) return;

          const nodes = rawWorkflow.nodes as Node[];
          if (nodes.length < 2) return;

          const xs = nodes.map((n) => n.position.x);
          const ys = nodes.map((n) => n.position.y);
          const centerX = (Math.min(...xs) + Math.max(...xs)) / 2;
          const centerY = (Math.min(...ys) + Math.max(...ys)) / 2;

          nodes.forEach((node) => {
            const scaledNode: Node = {
              ...node,
              position: {
                x:
                  Math.round(
                    (centerX + (node.position.x - centerX) * xScale) /
                      DEFAULT_GRID_SIZE,
                  ) * DEFAULT_GRID_SIZE,
                y:
                  Math.round(
                    (centerY + (node.position.y - centerY) * yScale) /
                      DEFAULT_GRID_SIZE,
                  ) * DEFAULT_GRID_SIZE,
              },
            };
            yNodes.set(node.id, yNodeConstructor(scaledNode));
          });
        });
      });
    },
    [rawWorkflows, yWorkflows, undoTrackerActionWrapper],
  );

  return {
    handleYLayoutChange,
    handleYSpacingChange,
  };
};
