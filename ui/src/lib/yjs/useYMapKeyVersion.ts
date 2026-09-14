import { useCallback, useRef, useSyncExternalStore } from "react";
import type * as Y from "yjs";

/**
 * Re-renders when the *keys* of a Y.Map change (added, replaced, deleted),
 * without serializing its contents.
 *
 * `useY` observes deeply and calls `toJSON()` on every change, which is far too
 * expensive for the workflows map — every node drag in any workflow would
 * serialize the whole project. But something has to observe that map: when a
 * workflow entry is deleted or replaced, Yjs raises no event on the child
 * nodes/edges maps, so a component watching only those keeps rendering a
 * detached Y.Map. Writes then go into a type that is no longer in the document
 * and are silently lost — including deletes, which is one of the ways an edge
 * survives being removed in the UI.
 */
export default <T>(yMap?: Y.Map<T>) => {
  const versionRef = useRef(0);

  const subscribe = useCallback(
    (onStoreChange: () => void) => {
      if (!yMap) return () => {};
      const handler = () => {
        versionRef.current += 1;
        onStoreChange();
      };
      yMap.observe(handler);
      return () => yMap.unobserve(handler);
    },
    [yMap],
  );

  const getSnapshot = useCallback(() => versionRef.current, []);

  return useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
};
