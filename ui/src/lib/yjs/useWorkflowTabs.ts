import { useCallback, useMemo, useState } from "react";

import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { Edge, Node } from "@flow/types";
import { isDefined } from "@flow/utils";

export default ({
  currentWorkflowId,
  rawWorkflows,
  handleCurrentWorkflowIdChange,
}: {
  currentWorkflowId: string;
  rawWorkflows: Record<string, string | Node[] | Edge[]>[];
  handleCurrentWorkflowIdChange: (id?: string) => void;
}) => {
  // This works as a semi-static base for the rest of the state in this hook.
  // Without this state (aka using rawWorkflows directly), performance drops
  // due to the state updating on every change to a node (which is a lot)
  const [workflows, setWorkflows] = useState<{ id: string; name: string }[]>(
    rawWorkflows.filter(isDefined).map((w2) => ({
      id: w2.id as string,
      name: w2.name as string,
    })),
  );

  const [openWorkflowIds, setOpenWorkflowIds] = useState<string[]>([
    DEFAULT_ENTRY_GRAPH_ID,
  ]);

  const currentWorkflowIndex = useMemo(
    () => workflows.findIndex((w) => w.id === currentWorkflowId),
    [currentWorkflowId, workflows],
  );

  const openWorkflows: {
    id: string;
    name: string;
  }[] = useMemo(
    () =>
      openWorkflowIds
        .map((owi) => workflows.find((w) => owi === w.id))
        .filter(isDefined),
    [workflows, openWorkflowIds],
  );

  const handleWorkflowOpen = useCallback(
    (workflowId: string) => {
      setOpenWorkflowIds((ids) => {
        handleCurrentWorkflowIdChange(workflowId);
        if (ids.includes(workflowId)) return ids;
        return [...ids, workflowId];
      });
    },
    [handleCurrentWorkflowIdChange],
  );

  const handleWorkflowClose = useCallback(
    (workflowId: string) => {
      setOpenWorkflowIds((ids) => {
        const index = ids.findIndex((id) => id === workflowId);
        const filteredIds = ids.filter((id) => id !== workflowId);
        const currentWorkflowIndex = openWorkflowIds.findIndex(
          (wid) => wid === currentWorkflowId,
        );
        if (
          workflowId !== DEFAULT_ENTRY_GRAPH_ID &&
          index === currentWorkflowIndex
        ) {
          handleCurrentWorkflowIdChange(ids[index - 1]);
        }
        return filteredIds;
      });
    },
    [openWorkflowIds, currentWorkflowId, handleCurrentWorkflowIdChange],
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
