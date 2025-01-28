import isEqual from "lodash-es/isEqual";
import { useCallback } from "react";

import type { Edge } from "@flow/types";

import { yEdgeConstructor } from "./conversions";
import type { YEdgesArray, YWorkflow } from "./types";

export default ({
  currentYWorkflow,
  undoTrackerActionWrapper,
}: {
  currentYWorkflow: YWorkflow;
  undoTrackerActionWrapper: (callback: () => void) => void;
}) => {
  const handleYEdgesUpdate = useCallback(
    (newEdges: Edge[]) =>
      undoTrackerActionWrapper(() => {
        const yEdges = currentYWorkflow?.get("edges") as
          | YEdgesArray
          | undefined;
        if (!yEdges) return;

        const e = yEdges.toJSON() as Edge[];

        if (isEqual(e, newEdges)) return;

        const newYEdges = newEdges.map((edge) => yEdgeConstructor(edge));

        yEdges.delete(0, e.length);
        yEdges.insert(0, newYEdges);
      }),
    [currentYWorkflow, undoTrackerActionWrapper],
  );
  return {
    handleYEdgesUpdate,
  };
};
