/**
 * Matches when the text before the cursor ends inside an open
 * `attributes["…` / `attributes['…` string literal (no closing quote yet).
 * Used to switch FlowExpr autocomplete to attribute-name suggestions.
 */
export const ATTRIBUTE_ACCESSOR_RE = /attributes\s*\[\s*["'][^"']*$/;

const ATTRIBUTE_ACCESSOR_PREFIX_RE = /attributes\s*\[\s*["']([^"']*)$/;

const IDENTIFIER_RE = /[a-zA-Z0-9_:.]/;
/** As above, minus the separators — used to find the end of a single segment. */
const IDENTIFIER_SEGMENT_RE = /[a-zA-Z0-9_]/;

export const isInsideAttributeAccessor = (textBeforeCursor: string): boolean =>
  ATTRIBUTE_ACCESSOR_RE.test(textBeforeCursor);

export type CompletionContext = {
  kind: "attribute" | "general";
  prefix: string;
  start: number;
  end: number;
  afterDot: boolean;
};

export const getCompletionContext = (
  text: string,
  caret: number,
): CompletionContext => {
  const cursor = Math.max(0, Math.min(caret, text.length));
  const before = text.substring(0, cursor);

  const attributeMatch = before.match(ATTRIBUTE_ACCESSOR_PREFIX_RE);
  if (attributeMatch) {
    const prefix = attributeMatch[1];
    let end = cursor;
    while (end < text.length && !/["'\n]/.test(text[end])) {
      end++;
    }
    return {
      kind: "attribute",
      prefix,
      start: cursor - prefix.length,
      end,
      afterDot: false,
    };
  }

  let start = cursor;
  while (start > 0 && IDENTIFIER_RE.test(text[start - 1])) {
    start--;
  }

  let end = cursor;
  while (end < text.length && IDENTIFIER_SEGMENT_RE.test(text[end])) {
    end++;
  }

  const word = text.substring(start, cursor);
  const lastDot = word.lastIndexOf(".");

  return {
    kind: "general",
    prefix: lastDot >= 0 ? word.substring(lastDot + 1) : word,
    start: lastDot >= 0 ? start + lastDot + 1 : start,
    end,
    afterDot: lastDot >= 0,
  };
};
