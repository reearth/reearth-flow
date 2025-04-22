import { useCallback, useEffect, useMemo, useState } from "react";

import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { Workflow } from "@flow/types";
import { isDefined } from "@flow/utils";

export default ({
  currentWorkflowId,
  rawWorkflows,
  setCurrentWorkflowId,
}: {
  currentWorkflowId: string;
  rawWorkflows: Workflow[];
  setCurrentWorkflowId: (id: string) => void;
}) => {
  const isMainWorkflow = useMemo(
    () => currentWorkflowId === DEFAULT_ENTRY_GRAPH_ID,
    [currentWorkflowId],
  );

  const [workflowNames, setWorkflowsNames] = useState(
    rawWorkflows.map((w) => ({ id: w.id, name: w.name })),
  );
  // This works as a semi-static base for the rest of the state in this hook.
  // Without this state (aka using rawWorkflows directly), performance drops
  // due to the state updating on every change to a node (which is a lot)
  useEffect(() => {
    if (rawWorkflows.length !== workflowNames.length) {
      setWorkflowsNames(rawWorkflows.map((w) => ({ id: w.id, name: w.name })));
    }
  }, [rawWorkflows.length, workflowNames.length]); // eslint-disable-line react-hooks/exhaustive-deps

  const workflows = useMemo(() => {
    return workflowNames.filter(isDefined).map((w2) => ({
      id: w2.id as string,
      name: w2.name as string,
    }));
  }, [workflowNames]);

  const handleCurrentWorkflowIdChange = useCallback(
    (id?: string) => {
      if (!id) return setCurrentWorkflowId(DEFAULT_ENTRY_GRAPH_ID);
      setCurrentWorkflowId(id);
    },
    [setCurrentWorkflowId],
  );

  const [openWorkflowIds, setOpenWorkflowIds] = useState<string[]>([
    DEFAULT_ENTRY_GRAPH_ID,
  ]);

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
    isMainWorkflow,
    handleWorkflowOpen,
    handleWorkflowClose,
    handleCurrentWorkflowIdChange,
    setWorkflowsNames,
    workflowNames,
  };
};
