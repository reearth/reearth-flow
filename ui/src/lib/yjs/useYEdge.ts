import isEqual from "lodash-es/isEqual";
import { useCallback } from "react";

import type { Edge } from "@flow/types";

import { YEdgesArray, YWorkflow } from "./workflowBuilder";

export default ({
  currentYWorkflow,
  undoTrackerActionWrapper,
}: {
  currentYWorkflow: YWorkflow;
  undoTrackerActionWrapper: (callback: () => void) => void;
}) => {
  const handleEdgesUpdate = useCallback(
    (newEdges: Edge[]) =>
      undoTrackerActionWrapper(() => {
        const yEdges = currentYWorkflow?.get("edges") as
          | YEdgesArray
          | undefined;
        if (!yEdges) return;

        const e = yEdges.toJSON() as Edge[];

        if (isEqual(e, newEdges)) return;

        yEdges.delete(0, e.length);
        yEdges.insert(0, newEdges);
      }),
    [currentYWorkflow, undoTrackerActionWrapper],
  );
  return {
    handleEdgesUpdate,
  };
};
