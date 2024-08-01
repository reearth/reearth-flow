import { useCallback, useMemo, useState } from "react";
import { Text as YText } from "yjs";

import { Edge, Node } from "@flow/types";

import { fromYjsText } from "./conversions";

export default ({
  workflowId,
  rawWorkflows,
  handleWorkflowIdChange,
}: {
  workflowId?: string;
  rawWorkflows: {
    [key: string]: YText | Node[] | Edge[];
  }[];
  handleWorkflowIdChange: (id?: string) => void;
}) => {
  const [workflows, setWorkflows] = useState<{ id: string; name: string }[]>(
    rawWorkflows.map(w2 => ({
      id: fromYjsText(w2?.id as YText),
      name: fromYjsText(w2?.name as YText),
    })),
  );

  const [openWorkflowIds, setOpenWorkflowIds] = useState<string[]>(
    workflows.filter(w => w.id === "main").map(w => w.id),
  );

  const currentWorkflowIndex = useMemo(
    () => workflows.findIndex(w => w.id === workflowId),
    [workflowId, workflows],
  );

  const openWorkflows: {
    id: string;
    name: string;
  }[] = useMemo(
    () => workflows.filter(w => openWorkflowIds.includes(w.id)),
    [workflows, openWorkflowIds],
  );

  const handleWorkflowOpen = useCallback(
    (workflowId: string) => {
      setOpenWorkflowIds(ids => {
        handleWorkflowIdChange(workflowId);
        if (ids.includes(workflowId)) return ids;
        return [...ids, workflowId];
      });
    },
    [handleWorkflowIdChange],
  );

  const handleWorkflowClose = useCallback(
    (workflowId: string) => {
      setOpenWorkflowIds(ids => ids.filter(id => id !== workflowId));
      if (workflowId !== "main") {
        handleWorkflowIdChange("main");
      }
    },
    [handleWorkflowIdChange],
  );

  return {
    workflows,
    openWorkflows,
    currentWorkflowIndex,
    setWorkflows,
    setOpenWorkflowIds,
    handleWorkflowOpen,
    handleWorkflowClose,
  };
};
