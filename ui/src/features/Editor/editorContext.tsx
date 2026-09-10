import {
  createContext,
  FC,
  MouseEvent,
  PropsWithChildren,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import type { Doc } from "yjs";

import type { YWorkflow } from "@flow/lib/yjs/types";
import { useFollowSync, type SpotlightFollow } from "@flow/lib/yjs";
import {
  NodeChange,
  type AwarenessEchoMap,
  type AwarenessSelection,
  type AwarenessSelectionsMap,
} from "@flow/types";

export type WorkflowVarAwareness = {
  onDialogOpen: () => void;
  onDialogClose: () => void;
  onFieldFocus: (variableId: string | null, field: string | null) => void;
  onEditStart: (variableId: string | null) => void;
};

export type EditorContextType = {
  isLocked: boolean;
  isReaderRestricted: boolean;
  canViewIntermediateData: boolean;
  onNodesChange?: (changes: NodeChange[]) => void;
  onNodeSettings?: (_e: MouseEvent | undefined, nodeId: string) => void;
  currentYWorkflow?: YWorkflow;
  undoTrackerActionWrapper?: (
    callback: () => void,
    originPrepend?: string,
  ) => void;
  awarenessSelectionsMap?: AwarenessSelectionsMap;
  awarenessEchoMap?: AwarenessEchoMap;
  onEchoElement?: (key: string, active: boolean) => void;
  spotlightFollow?: SpotlightFollow;
  yDoc?: Doc | null;
  workflowVarAwareness?: WorkflowVarAwareness;
  staleNodeIds?: Set<string>;
};

const EditorContext = createContext<EditorContextType | undefined>(undefined);

export const EditorProvider: FC<
  PropsWithChildren<{ value: EditorContextType }>
> = ({ children, value }) => (
  <EditorContext.Provider value={value}>{children}</EditorContext.Provider>
);

export const useEditorContext = (): EditorContextType => {
  const ctx = useContext(EditorContext);
  if (!ctx) {
    throw new Error("Could not find EditorProvider");
  }

  return ctx;
};

/**
 * Editing is disabled either because the project is locked or because the
 * current user only has read access.
 */
export const useIsReadOnly = (): boolean => {
  const { isLocked, isReaderRestricted } = useEditorContext();
  return isLocked || isReaderRestricted;
};

export const useAwarenessNodeSelections = (
  nodeId: string,
): AwarenessSelection[] => {
  const { awarenessSelectionsMap } = useEditorContext();
  return awarenessSelectionsMap?.[nodeId] ?? [];
};

const NO_USERS: AwarenessSelection[] = [];

/**
 * Who else is currently on this UI element. Pair with `awarenessEchoStyles` to
 * ring it in their colour.
 *
 * Tolerates a missing provider so components can also render in Storybook and
 * in tests that do not set up an editor.
 */
export const useAwarenessEcho = (
  key: string | undefined,
): AwarenessSelection[] => {
  const ctx = useContext(EditorContext);
  if (!key) return NO_USERS;
  return ctx?.awarenessEchoMap?.[key] ?? NO_USERS;
};

/**
 * Broadcast that the local user is on this element while `isActive` holds.
 * Clears on unmount, so a dropdown closing or a row unmounting is enough.
 */
export const useEchoPresence = (
  key: string | undefined,
  isActive: boolean,
): void => {
  const onEchoElement = useContext(EditorContext)?.onEchoElement;

  useEffect(() => {
    if (!key || !isActive || !onEchoElement) return;
    onEchoElement(key, true);
    return () => onEchoElement(key, false);
  }, [key, isActive, onEchoElement]);
};

/** Whether the spotlighted user is currently on this element. */
const useSpotlightOnElement = (key: string) => {
  const spotlightFollow = useContext(EditorContext)?.spotlightFollow;
  return {
    isFollowing: spotlightFollow?.isFollowing ?? false,
    isOn: spotlightFollow?.activeElements.includes(key) ?? false,
  };
};

/**
 * Makes an already-stateful dropdown/menu/popover participate in awareness:
 * broadcasts while open, and opens when the spotlighted user opens it.
 *
 * The echo channel does double duty here — the same `activeElements` entry that
 * rings the element for everyone else is what tells a follower to open it, so
 * no per-dropdown awareness field is needed.
 *
 * Use this when the component already owns its open state; use
 * `useEchoDropdown` when it does not.
 */
export const useEchoDropdownSync = (
  key: string,
  open: boolean,
  setOpen: (open: boolean) => void,
) => {
  const { isFollowing, isOn } = useSpotlightOnElement(key);

  useEchoPresence(key, open);

  const setOpenRef = useRef(setOpen);
  setOpenRef.current = setOpen;

  const handleFollow = useCallback((next: true | null) => {
    setOpenRef.current(!!next);
  }, []);

  useFollowSync<true>({
    active: isFollowing,
    follow: isOn ? true : null,
    local: open ? true : null,
    onFollow: handleFollow,
  });
};

/** `useEchoDropdownSync` for a dropdown that has no open state of its own. */
export const useEchoDropdown = (key: string) => {
  const [open, setOpen] = useState(false);
  useEchoDropdownSync(key, open, setOpen);
  return { open, onOpenChange: setOpen };
};

/**
 * Echo for an element the user focuses into (an input, a table cell). Spread
 * `focusProps`; capture-phase so focus on a nested control still counts.
 */
export const useEchoFocus = (key: string) => {
  const [focused, setFocused] = useState(false);
  useEchoPresence(key, focused);
  const users = useAwarenessEcho(key);

  const focusProps = useMemo(
    () => ({
      onFocusCapture: () => setFocused(true),
      onBlurCapture: () => setFocused(false),
    }),
    [],
  );

  return { users, focusProps };
};

/**
 * Echo for a hoverable item (menu entry, list row). Spread `hoverProps` on the
 * element and style it from `users` with `awarenessEchoStyles`.
 */
export const useEchoHover = (key: string) => {
  const [hovered, setHovered] = useState(false);
  useEchoPresence(key, hovered);
  const users = useAwarenessEcho(key);

  const hoverProps = useMemo(
    () => ({
      onMouseEnter: () => setHovered(true),
      onMouseLeave: () => setHovered(false),
    }),
    [],
  );

  return { users, hoverProps };
};
