import { useCallback, useMemo, useState } from "react";

import type { Workflow } from "@flow/types";
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
  const mainWorkflow = useMemo(
    () => rawWorkflows.find((rw) => rw.isMain),
    [rawWorkflows],
  );

  const isMainWorkflow = useMemo(
    () =>
      rawWorkflows.find((rw) => rw.id === mainWorkflow?.id)?.isMain ?? false,
    [mainWorkflow, rawWorkflows],
  );

  // This works as a semi-static base for the rest of the state in this hook.
  // Without this state (aka using rawWorkflows directly), performance drops
  // due to the state updating on every change to a node (which is a lot)
  const workflows = useMemo(
    () =>
      rawWorkflows.filter(isDefined).map((w2) => ({
        id: w2.id as string,
        name: w2.name as string,
      })),
    [rawWorkflows.length], // eslint-disable-line react-hooks/exhaustive-deps
  );

  const handleCurrentWorkflowIdChange = useCallback(
    (id?: string) => {
      if (!id)
        return setCurrentWorkflowId(mainWorkflow?.id ?? rawWorkflows[0].id);
      setCurrentWorkflowId(id);
    },
    [mainWorkflow?.id, rawWorkflows, setCurrentWorkflowId],
  );

  const [openWorkflowIds, setOpenWorkflowIds] = useState<string[]>([
    mainWorkflow?.id ?? rawWorkflows[0].id,
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
        if (workflowId !== mainWorkflow?.id && index === currentWorkflowIndex) {
          handleCurrentWorkflowIdChange(ids[index - 1]);
        }
        return filteredIds;
      });
    },
    [
      openWorkflowIds,
      mainWorkflow?.id,
      currentWorkflowId,
      handleCurrentWorkflowIdChange,
    ],
  );

  return {
    openWorkflows,
    isMainWorkflow,
    handleWorkflowOpen,
    handleWorkflowClose,
    handleCurrentWorkflowIdChange,
  };
};
