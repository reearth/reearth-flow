import { useCallback, useEffect, useMemo, useState } from "react";

import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import type { Workflow } from "@flow/types";
import {
  buildWorkflowParentMap,
  getWorkflowLineage,
  isDefined,
  isWorkflowDescendant,
} from "@flow/utils";
import type { WorkflowParentMap } from "@flow/utils";

export type OpenWorkflow = {
  id: string;
  name: string;
  depth: number;
  parentId?: string;
};

export default ({
  currentWorkflowId,
  rawWorkflows,
  setCurrentWorkflowId,
}: {
  currentWorkflowId: string;
  rawWorkflows: Workflow[];
  setCurrentWorkflowId: (id: string) => void;
}) => {
  const isMainWorkflow = useMemo(() => {
    return currentWorkflowId === DEFAULT_ENTRY_GRAPH_ID;
  }, [currentWorkflowId]);

  const [workflowNames, setWorkflowsNames] = useState(
    rawWorkflows.map((w) => ({ id: w.id, name: w.name })),
  );
  // Length check is used to for adding and removing workflows.
  // This works as a semi-static base for the rest of the state in this hook.
  // Without this state (aka using rawWorkflows directly), performance drops
  // due to the state updating on every change to a node (which is a lot)
  useEffect(() => {
    if (rawWorkflows.length !== workflowNames.length) {
      setWorkflowsNames(rawWorkflows.map((w) => ({ id: w.id, name: w.name })));
    }
  }, [rawWorkflows.length, workflowNames.length]); // eslint-disable-line react-hooks/exhaustive-deps

  // rawWorkflows changes on every node edit, but nesting only changes when a
  // subworkflow node is added or removed. Keying on the serialised map keeps
  // the parent map — and everything derived from it — stable in between.
  const parentMapKey = useMemo(
    () => JSON.stringify(buildWorkflowParentMap(rawWorkflows)),
    [rawWorkflows],
  );
  const parentMap: WorkflowParentMap = useMemo(
    () => JSON.parse(parentMapKey),
    [parentMapKey],
  );

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

  // Ordered depth-first so each subworkflow sits directly under the workflow
  // it is nested in; siblings keep the order they were opened in.
  const openWorkflows: OpenWorkflow[] = useMemo(() => {
    const open = openWorkflowIds
      .map((owi) => workflows.find((w) => owi === w.id))
      .filter(isDefined);
    const openIds = new Set(open.map((w) => w.id));

    const nearestOpenAncestor = (id: string) =>
      getWorkflowLineage(parentMap, id)
        .slice(0, -1)
        .reverse()
        .find((ancestorId) => openIds.has(ancestorId));

    const roots: typeof open = [];
    const children = new Map<string, typeof open>();
    const parents = new Map<string, string>();
    for (const workflow of open) {
      const parentId = nearestOpenAncestor(workflow.id);
      if (!parentId) {
        roots.push(workflow);
        continue;
      }
      parents.set(workflow.id, parentId);
      children.set(parentId, [...(children.get(parentId) ?? []), workflow]);
    }

    const ordered: OpenWorkflow[] = [];
    const visited = new Set<string>();
    const visit = (workflow: (typeof open)[number], depth: number) => {
      if (visited.has(workflow.id)) return;
      visited.add(workflow.id);
      ordered.push({ ...workflow, depth, parentId: parents.get(workflow.id) });
      children.get(workflow.id)?.forEach((child) => visit(child, depth + 1));
    };
    roots.forEach((root) => visit(root, 0));
    open
      .filter((workflow) => !visited.has(workflow.id))
      .forEach((root) => {
        parents.delete(root.id);
        visit(root, 0);
      });
    return ordered;
  }, [workflows, openWorkflowIds, parentMap]);

  const getLineage = useCallback(
    (workflowId: string) => getWorkflowLineage(parentMap, workflowId),
    [parentMap],
  );

  // Opening a nested subworkflow (from search, a diagnostic, a spotlighted
  // user) opens every workflow above it too, so the list shows where it is.
  const handleWorkflowOpen = useCallback(
    (workflowId: string) => {
      setOpenWorkflowIds((ids) => {
        handleCurrentWorkflowIdChange(workflowId);
        const missing = getWorkflowLineage(parentMap, workflowId).filter(
          (id) => !ids.includes(id),
        );
        if (missing.length === 0) return ids;
        return [...ids, ...missing];
      });
    },
    [handleCurrentWorkflowIdChange, parentMap],
  );

  // Closing a workflow closes the subworkflows nested in it. If the user was
  // inside any of them, they land on the closest workflow still open above.
  const handleWorkflowClose = useCallback(
    (workflowId: string) => {
      setOpenWorkflowIds((ids) => {
        if (workflowId === DEFAULT_ENTRY_GRAPH_ID) {
          return ids.filter((id) => id !== workflowId);
        }
        const remaining = ids.filter(
          (id) =>
            id !== workflowId &&
            !isWorkflowDescendant(parentMap, id, workflowId),
        );
        if (!remaining.includes(currentWorkflowId)) {
          const fallback = getWorkflowLineage(parentMap, workflowId)
            .slice(0, -1)
            .reverse()
            .find((id) => remaining.includes(id));
          handleCurrentWorkflowIdChange(fallback);
        }
        return remaining;
      });
    },
    [currentWorkflowId, handleCurrentWorkflowIdChange, parentMap],
  );

  return {
    isMainWorkflow,
    openWorkflows,
    openWorkflowIds,
    workflowNames,
    getWorkflowLineage: getLineage,
    handleWorkflowOpen,
    handleWorkflowClose,
    handleCurrentWorkflowIdChange,
    setWorkflowsNames,
  };
};
