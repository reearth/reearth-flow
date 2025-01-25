import { useMemo } from "react";

import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";

export default (currentWorkflowId: string | undefined) => {
  return useMemo(() => {
    return currentWorkflowId === DEFAULT_ENTRY_GRAPH_ID;
  }, [currentWorkflowId]);
};
