import { useReactFlow } from "@xyflow/react";
import { useCallback, useEffect, useRef, useState } from "react";
import type { Awareness } from "y-protocols/awareness";

import type { AwarenessUser } from "@flow/types";

export default ({
  yAwareness,
  users,
  currentWorkflowId,
  openWorkflowIds,
  handleWorkflowOpen,
  handleWorkflowClose,
  getWorkflowLineage,
}: {
  yAwareness: Awareness;
  users: Record<string, AwarenessUser>;
  currentWorkflowId: string;
  openWorkflowIds: string[];
  handleWorkflowOpen: (workflowId: string) => void;
  handleWorkflowClose: (workflowId: string) => void;
  getWorkflowLineage?: (workflowId: string) => string[];
}) => {
  const { setViewport } = useReactFlow();
  const [spotlightUserClientId, setSpotlightUserClientId] = useState<
    number | null
  >(null);
  const spotlightUser = spotlightUserClientId
    ? users[spotlightUserClientId]
    : null;
  const spotlightUserCurrentWorkflowId = spotlightUser?.currentWorkflowId;
  const spotlightUserOpenWorkflowIds = spotlightUser?.openWorkflowIds;
  const spotlightUserViewport = spotlightUser?.viewport;
  const workflowsOpenedBySpotlight = useRef<Set<string>>(new Set());
  const prevSpotlightUserOpenWorkflowIds = useRef<string[] | undefined>(
    undefined,
  );

  useEffect(() => {
    if (!spotlightUserCurrentWorkflowId || !spotlightUserOpenWorkflowIds)
      return;
    if (spotlightUserCurrentWorkflowId !== currentWorkflowId) {
      // Opening a nested workflow opens its parents too; track those as well
      // so they close again when the spotlighted user closes them.
      const opening = getWorkflowLineage?.(spotlightUserCurrentWorkflowId) ?? [
        spotlightUserCurrentWorkflowId,
      ];
      opening
        .filter((id) => !openWorkflowIds.includes(id))
        .forEach((id) => workflowsOpenedBySpotlight.current.add(id));
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
    getWorkflowLineage,
  ]);

  useEffect(() => {
    if (!spotlightUserViewport) return;
    setViewport(
      {
        x: spotlightUserViewport.x,
        y: spotlightUserViewport.y,
        zoom: spotlightUserViewport.zoom,
      },
      { duration: 100 },
    );
  }, [spotlightUserViewport, setViewport]);

  useEffect(() => {
    if (!spotlightUserClientId || !spotlightUserOpenWorkflowIds) return;

    prevSpotlightUserOpenWorkflowIds.current = spotlightUserOpenWorkflowIds;
  }, [spotlightUserClientId, spotlightUserOpenWorkflowIds]);

  const handleSpotlightUserSelect = useCallback((clientId: number) => {
    setSpotlightUserClientId(clientId);
  }, []);

  const handleSpotlightUserDeselect = useCallback(() => {
    if (spotlightUser && spotlightUserClientId) {
      setSpotlightUserClientId(null);
    }
  }, [spotlightUser, spotlightUserClientId]);

  useEffect(() => {
    if (Object.keys(users).length === 0) return;
    yAwareness.setLocalStateField("currentWorkflowId", currentWorkflowId);
  }, [currentWorkflowId, yAwareness, users]);

  useEffect(() => {
    if (Object.keys(users).length === 0) return;
    yAwareness.setLocalStateField("openWorkflowIds", openWorkflowIds);
  }, [openWorkflowIds, yAwareness, users]);

  return {
    spotlightUser,
    spotlightUserClientId,
    handleSpotlightUserSelect,
    handleSpotlightUserDeselect,
  };
};
