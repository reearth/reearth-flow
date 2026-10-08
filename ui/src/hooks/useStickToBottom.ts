import { useCallback, useLayoutEffect, useRef } from "react";

// Sub-pixel scroll offsets on scaled displays mean "at the bottom" is rarely
// an exact zero, so anything within a few pixels counts.
const BOTTOM_THRESHOLD_PX = 8;

/**
 * Keeps a scroll container pinned to its bottom while new content arrives, the
 * way a terminal does. The pin is released as soon as the user scrolls up and
 * taken again once they scroll back down, so reading older entries is never
 * interrupted by newer ones.
 *
 * `contentKey` should change whenever the content may have grown (e.g. a row
 * count). Attach `ref` and `onScroll` to the element that scrolls.
 */
export default function useStickToBottom<T extends HTMLElement>(
  contentKey: unknown,
) {
  const ref = useRef<T>(null);
  // Starts pinned so the newest entries are what a freshly opened view shows.
  const isPinned = useRef(true);

  const onScroll = useCallback(() => {
    const el = ref.current;
    if (!el) return;
    isPinned.current =
      el.scrollHeight - el.scrollTop - el.clientHeight <= BOTTOM_THRESHOLD_PX;
  }, []);

  // Layout effect, so the jump happens before paint and the new rows never
  // flash in at the old position first.
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el || !isPinned.current) return;
    el.scrollTop = el.scrollHeight;
  }, [contentKey]);

  return { ref, onScroll };
}
