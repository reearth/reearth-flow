import isEqual from "lodash/isEqual";
import { useCallback } from "react";

import type { Edge } from "@flow/types";

import { YEdgesArray, YWorkflow } from "./workflowBuilder";

export default (workflow: YWorkflow) => {
  const handleEdgesUpdate = useCallback(
    (newEdges: Edge[]) => {
      const yEdges = workflow?.get("edges") as YEdgesArray | undefined;
      if (!yEdges) return;

      const e = yEdges.toJSON() as Edge[];

      if (isEqual(e, newEdges)) return;

      yEdges.delete(0, e.length);
      yEdges.insert(0, newEdges);
    },
    [workflow],
  );
  return {
    handleEdgesUpdate,
  };
};
