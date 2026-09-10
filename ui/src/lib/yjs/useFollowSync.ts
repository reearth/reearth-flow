import { useEffect, useRef } from "react";

/**
 * Mirrors one slice of a spotlighted user's UI state into the follower's own
 * local state.
 *
 * Two things are deliberately asymmetric:
 * - While following, the hook tracks whether the follow is what opened a thing,
 *   so the spotlighted user closing it closes it here too, but anything the
 *   follower already had open is left alone.
 * - When the follow session itself ends (`active` goes false, e.g. the follower
 *   clicked something and dropped the spotlight) nothing is closed. The
 *   follower is driving from here on, and yanking a dialog out from under a
 *   click is worse than leaving it open.
 *
 * `follow` must be referentially stable while unchanged; object-valued slices
 * are memoized on a string digest in `useSpotlightUser` for exactly this reason.
 */
export default function useFollowSync<T>({
  active,
  follow,
  local,
  onFollow,
  isSame = Object.is,
}: {
  /** Whether a user is currently spotlighted. */
  active: boolean;
  /** The spotlighted user's value, or null when they have nothing open. */
  follow: T | null;
  /** The follower's current value for the same slice. */
  local: T | null;
  /** Applies (or clears, on null) the followed value locally. */
  onFollow: (next: T | null) => void;
  isSame?: (a: T, b: T) => boolean;
}) {
  const openedByFollowRef = useRef(false);

  // Read through refs so the effect only reruns when the follow itself changes.
  const localRef = useRef(local);
  localRef.current = local;
  const onFollowRef = useRef(onFollow);
  onFollowRef.current = onFollow;
  const isSameRef = useRef(isSame);
  isSameRef.current = isSame;

  useEffect(() => {
    if (!active) {
      openedByFollowRef.current = false;
      return;
    }

    const current = localRef.current;

    if (follow !== null && follow !== undefined) {
      if (current !== null && isSameRef.current(current, follow)) return;
      openedByFollowRef.current = true;
      onFollowRef.current(follow);
      return;
    }

    if (openedByFollowRef.current) {
      openedByFollowRef.current = false;
      if (current !== null) onFollowRef.current(null);
    }
  }, [active, follow]);
}
