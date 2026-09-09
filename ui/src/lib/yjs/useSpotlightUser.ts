import { useReactFlow } from "@xyflow/react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { Awareness } from "y-protocols/awareness";

import type {
  AwarenessDialog,
  AwarenessNodePicker,
  AwarenessSubEditor,
  AwarenessUser,
} from "@flow/types";

/**
 * The slice of a spotlighted user's state that the follower's own UI mirrors.
 * All fields are null while nobody is spotlighted, which is what drives the
 * local sync effects to close whatever the follow opened.
 */
export type SpotlightFollow = {
  /** Whether a user is currently spotlighted at all. */
  isFollowing: boolean;
  openNodeId: string | null;
  dialog: AwarenessDialog | null;
  subEditor: AwarenessSubEditor | null;
  nodePicker: AwarenessNodePicker | null;
  versionSnapshot: number | null;
  /** Echo keys the spotlighted user is on; drives dropdown/menu follow. */
  activeElements: string[];
};

const NO_FOLLOW: SpotlightFollow = {
  isFollowing: false,
  openNodeId: null,
  dialog: null,
  subEditor: null,
  nodePicker: null,
  versionSnapshot: null,
  activeElements: [],
};

export default ({
  yAwareness,
  users,
  currentWorkflowId,
  openWorkflowIds,
  handleWorkflowOpen,
  handleWorkflowClose,
}: {
  yAwareness: Awareness;
  users: Record<string, AwarenessUser>;
  currentWorkflowId: string;
  openWorkflowIds: string[];
  handleWorkflowOpen: (workflowId: string) => void;
  handleWorkflowClose: (workflowId: string) => void;
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

  // Mirrors the spotlighted user's dialog state. Memoized on the primitive
  // fields so a cursor move on the spotlighted user doesn't retrigger the
  // follower's sync effects.
  const spotlightSubEditor = spotlightUser?.openSubEditor ?? null;
  const spotlightDialog = spotlightUser?.openDialog ?? null;
  const spotlightOpenNodeId = spotlightUser?.openNodeId ?? null;
  const spotlightNodePicker = spotlightUser?.openNodePicker ?? null;
  const spotlightVersionSnapshot =
    spotlightUser?.selectedVersionSnapshot ?? null;
  const spotlightActiveElements = spotlightUser?.activeElements;
  const activeElementsKey = spotlightActiveElements?.join(",") ?? "";
  const activeElementsRef = useRef<string[]>([]);
  activeElementsRef.current = spotlightActiveElements ?? [];

  const subEditorKey = spotlightSubEditor
    ? `${spotlightSubEditor.kind}:${spotlightSubEditor.fieldId}`
    : null;
  const nodePickerKey = spotlightNodePicker
    ? `${spotlightNodePicker.nodeType}:${spotlightNodePicker.position.x}:${spotlightNodePicker.position.y}`
    : null;

  const subEditorRef = useRef(spotlightSubEditor);
  subEditorRef.current = spotlightSubEditor;
  const nodePickerRef = useRef(spotlightNodePicker);
  nodePickerRef.current = spotlightNodePicker;

  const spotlightFollow = useMemo<SpotlightFollow>(() => {
    if (!spotlightUserClientId) return NO_FOLLOW;
    return {
      isFollowing: true,
      openNodeId: spotlightOpenNodeId,
      dialog: spotlightDialog,
      subEditor: subEditorRef.current,
      nodePicker: nodePickerRef.current,
      versionSnapshot: spotlightVersionSnapshot,
      activeElements: activeElementsRef.current,
    };
    // subEditor/nodePicker are read through refs and keyed by their string
    // digests so identity churn from awareness updates doesn't leak out.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [
    spotlightUserClientId,
    spotlightOpenNodeId,
    spotlightDialog,
    spotlightVersionSnapshot,
    subEditorKey,
    nodePickerKey,
    activeElementsKey,
  ]);

  const handleSpotlightUserSelect = useCallback((clientId: number) => {
    setSpotlightUserClientId(clientId);
  }, []);

  const handleSpotlightUserDeselect = useCallback(() => {
    if (spotlightUser && spotlightUserClientId) {
      setSpotlightUserClientId(null);
    }
  }, [spotlightUser, spotlightUserClientId]);

  useEffect(() => {
    yAwareness.setLocalStateField(
      "followingClientId",
      spotlightUserClientId ?? null,
    );
  }, [spotlightUserClientId, yAwareness]);

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
    spotlightFollow,
    handleSpotlightUserSelect,
    handleSpotlightUserDeselect,
  };
};
