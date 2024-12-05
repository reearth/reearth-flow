import { useCallback, useMemo, useState } from "react";

import { Edge, Node } from "@flow/types";
import { isDefined } from "@flow/utils";

export default ({
  workflowId,
  rawWorkflows,
  handleWorkflowIdChange,
}: {
  workflowId?: string;
  rawWorkflows: Record<string, string | Node[] | Edge[]>[];
  handleWorkflowIdChange: (id?: string) => void;
}) => {
  const [workflows, setWorkflows] = useState<{ id: string; name: string }[]>(
    rawWorkflows.filter(isDefined).map((w2) => ({
      id: w2.id as string,
      name: w2.name as string,
    })),
  );

  const [openWorkflowIds, setOpenWorkflowIds] = useState<string[]>(
    workflows.filter((w) => w.id === "main").map((w) => w.id),
  );

  const currentWorkflowIndex = useMemo(
    () => workflows.findIndex((w) => w.id === workflowId),
    [workflowId, workflows],
  );

  const openWorkflows: {
    id: string;
    name: string;
  }[] = useMemo(
    () => workflows.filter((w) => openWorkflowIds.includes(w.id)),
    [workflows, openWorkflowIds],
  );

  const handleWorkflowOpen = useCallback(
    (workflowId: string) => {
      setOpenWorkflowIds((ids) => {
        handleWorkflowIdChange(workflowId);
        if (ids.includes(workflowId)) return ids;
        return [...ids, workflowId];
      });
    },
    [handleWorkflowIdChange],
  );

  const handleWorkflowClose = useCallback(
    (workflowId: string) => {
      setOpenWorkflowIds((ids) => {
        const index = ids.findIndex((id) => id === workflowId);
        const filteredIds = ids.filter((id) => id !== workflowId);
        if (workflowId !== "main" && index === currentWorkflowIndex) {
          handleWorkflowIdChange(ids[index - 1]);
        }
        return filteredIds;
      });
    },
    [currentWorkflowIndex, handleWorkflowIdChange],
  );

  return {
    openWorkflows,
    currentWorkflowIndex,
    setWorkflows,
    setOpenWorkflowIds,
    handleWorkflowOpen,
    handleWorkflowClose,
  };
};
