/**
 * Matches when the text before the cursor ends inside an open
 * `<name>["…` / `<name>['…` string literal (no closing quote yet). The leading
 * guard keeps `myenv["` from reading as an `env` accessor.
 */
const accessorRe = (name: string) =>
  new RegExp(`(?:^|[^a-zA-Z0-9_$])${name}\\s*\\[\\s*["'][^"']*$`);

const accessorPrefixRe = (name: string) =>
  new RegExp(`(?:^|[^a-zA-Z0-9_$])${name}\\s*\\[\\s*["']([^"']*)$`);

export const ATTRIBUTE_ACCESSOR_RE = accessorRe("attributes");
export const ENV_ACCESSOR_RE = accessorRe("env");

const ATTRIBUTE_ACCESSOR_PREFIX_RE = accessorPrefixRe("attributes");
const ENV_ACCESSOR_PREFIX_RE = accessorPrefixRe("env");

const IDENTIFIER_RE = /[a-zA-Z0-9_:.]/;
/** As above, minus the separators — used to find the end of a single segment. */
const IDENTIFIER_SEGMENT_RE = /[a-zA-Z0-9_]/;

export const isInsideAttributeAccessor = (textBeforeCursor: string): boolean =>
  ATTRIBUTE_ACCESSOR_RE.test(textBeforeCursor);

export const isInsideEnvAccessor = (textBeforeCursor: string): boolean =>
  ENV_ACCESSOR_RE.test(textBeforeCursor);

export type CompletionContext = {
  kind: "attribute" | "env" | "general";
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

  const accessors = [
    { kind: "attribute", re: ATTRIBUTE_ACCESSOR_PREFIX_RE },
    { kind: "env", re: ENV_ACCESSOR_PREFIX_RE },
  ] as const;

  for (const { kind, re } of accessors) {
    const match = before.match(re);
    if (!match) continue;

    const prefix = match[1];
    let end = cursor;
    while (end < text.length && !/["'\n]/.test(text[end])) {
      end++;
    }
    return {
      kind,
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
