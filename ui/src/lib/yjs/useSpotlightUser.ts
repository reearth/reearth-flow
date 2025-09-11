import { useEffect, useRef } from "react";

import type { AwarenessUser } from "@flow/types";

export default ({
  spotlightUser,
  currentWorkflowId,
  spotlightUserClientId,
  openWorkflowIds,
  handleWorkflowOpen,
  handleWorkflowClose,
}: {
  spotlightUser: AwarenessUser | null;
  currentWorkflowId: string;
  spotlightUserClientId: number | null;
  openWorkflowIds: string[];
  handleWorkflowOpen: (workflowId: string) => void;
  handleWorkflowClose: (workflowId: string) => void;
}) => {
  const spotlightUserCurrentWorkflowId = spotlightUser?.currentWorkflowId;
  const spotlightUserOpenWorkflowIds = spotlightUser?.openWorkflowIds;

  const workflowsOpenedBySpotlight = useRef<Set<string>>(new Set());
  const prevSpotlightUserOpenWorkflowIds = useRef<string[] | undefined>(
    undefined,
  );
  useEffect(() => {
    if (!spotlightUserCurrentWorkflowId || !spotlightUserOpenWorkflowIds)
      return;
    if (spotlightUserCurrentWorkflowId !== currentWorkflowId) {
      if (!openWorkflowIds.includes(spotlightUserCurrentWorkflowId)) {
        workflowsOpenedBySpotlight.current.add(spotlightUserCurrentWorkflowId);
      }
      handleWorkflowOpen(spotlightUserCurrentWorkflowId);
    }

    const prevIds = prevSpotlightUserOpenWorkflowIds.current;
    if (prevIds) {
      const closedWorkflowIds = prevIds.filter(
        (id) => !spotlightUserOpenWorkflowIds.includes(id),
      );

      closedWorkflowIds.forEach((workflowId) => {
        if (
          openWorkflowIds.includes(workflowId) &&
          workflowsOpenedBySpotlight.current.has(workflowId)
        ) {
          handleWorkflowClose(workflowId);
          workflowsOpenedBySpotlight.current.delete(workflowId);
        }
      });
    }
  }, [
    spotlightUserCurrentWorkflowId,
    currentWorkflowId,
    openWorkflowIds,
    spotlightUserOpenWorkflowIds,
    handleWorkflowOpen,
    handleWorkflowClose,
  ]);

  useEffect(() => {
    if (!spotlightUserClientId || !spotlightUserOpenWorkflowIds) return;

    prevSpotlightUserOpenWorkflowIds.current = spotlightUserOpenWorkflowIds;
  }, [spotlightUserClientId, spotlightUserOpenWorkflowIds]);
};
