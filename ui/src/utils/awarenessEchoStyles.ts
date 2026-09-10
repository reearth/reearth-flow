import type { AwarenessSelection } from "@flow/types";

/**
 * Ring for an element another user is on.
 *
 * Uses an inset box-shadow rather than a border so highlighting a list row or
 * menu item costs no layout — the row must not jump when someone hovers it.
 */
export const awarenessEchoStyles = (
  users: AwarenessSelection[] | null | undefined,
): React.CSSProperties | undefined =>
  users?.length
    ? {
        boxShadow: `inset 0 0 0 2px ${users[0].color}`,
        borderRadius: "4px",
      }
    : undefined;

/**
 * Presence marker for a row in a selectable list.
 *
 * A left stripe rather than the full ring `awarenessEchoStyles` gives: list rows
 * carry their own selected background, and a ring around one reads as "this row
 * is selected in their colour" instead of "they are also here". Follow syncs
 * selection, so the two markers land on the same row constantly and must stay
 * visually distinct.
 */
export const awarenessEchoStripe = (
  users: AwarenessSelection[] | null | undefined,
): React.CSSProperties | undefined =>
  users?.length
    ? { boxShadow: `inset 3px 0 0 0 ${users[0].color}` }
    : undefined;
