/**
 * Matches when the text before the cursor ends inside an open
 * `attributes["…` / `attributes['…` string literal (no closing quote yet).
 * Used to switch FlowExpr autocomplete to attribute-name suggestions.
 */
export const ATTRIBUTE_ACCESSOR_RE = /attributes\s*\[\s*["'][^"']*$/;

export const isInsideAttributeAccessor = (textBeforeCursor: string): boolean =>
  ATTRIBUTE_ACCESSOR_RE.test(textBeforeCursor);
