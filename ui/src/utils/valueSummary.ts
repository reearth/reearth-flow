/**
 * Describing arbitrary feature values without serializing all of them.
 *
 * Intermediate-data features carry values with no useful upper bound — mesh
 * face lists, point-cloud segments, per-corner UV. Anything that reaches for
 * `JSON.stringify` on every value it is handed will eventually be handed one of
 * those, so the entry points here measure first and summarize when measuring
 * says to.
 */

/** Leaf count past which a value is summarized rather than serialized. */
export const LARGE_VALUE_THRESHOLD = 100;

/** How many array items the inline preview expands. */
const ARRAY_PREVIEW_ITEMS = 1;

/** Nesting depth beyond which `stringifyItem` collapses to a shape note. */
const MAX_STRINGIFY_DEPTH = 3;

/** Array items rendered at any one depth by `stringifyItem`. */
const MAX_ARRAY_ITEMS = 3;

/** Object keys previewed by `summarizeValue`. */
const OBJECT_PREVIEW_KEYS = 8;

/** Resolve a value that might be a JSON string into its parsed form. */
export function resolveValue(value: unknown): unknown {
  if (typeof value === "string") {
    try {
      const parsed = JSON.parse(value);
      if (typeof parsed === "object" && parsed !== null) return parsed;
    } catch {
      // Not valid JSON
    }
  }
  return value;
}

/**
 * Estimate the number of leaf nodes in a value, stopping once it is clearly
 * large.
 *
 * `seen` guards against a value that references itself. Features come out of
 * `JSON.parse` and so cannot be cyclic, but this runs over whatever a caller
 * hands it, and overflowing the stack here would pre-empt the `try`/`catch`
 * the callers below rely on.
 */
export function estimateSize(
  value: unknown,
  seen = new WeakSet<object>(),
): number {
  const resolved = resolveValue(value);
  if (resolved == null || typeof resolved !== "object") return 1;
  if (seen.has(resolved)) return 1;
  seen.add(resolved);

  if (Array.isArray(resolved)) {
    // For large arrays, just use length — don't recurse into every element
    if (resolved.length > LARGE_VALUE_THRESHOLD) return resolved.length;
    let sum = 0;
    for (const item of resolved) {
      sum += estimateSize(item, seen);
      if (sum > LARGE_VALUE_THRESHOLD) return sum;
    }
    return sum;
  }
  const entries = Object.entries(resolved);
  if (entries.length > LARGE_VALUE_THRESHOLD) return entries.length;
  let sum = 0;
  for (const [, v] of entries) {
    sum += estimateSize(v, seen);
    if (sum > LARGE_VALUE_THRESHOLD) return sum;
  }
  return sum;
}

export function isLargeValue(value: unknown): boolean {
  return estimateSize(value) > LARGE_VALUE_THRESHOLD;
}

/**
 * Stringify a value with depth limiting (used for representative items in
 * previews). Beyond maxDepth, nested structures are shown as `Array(N)` /
 * `Object(N keys)`.
 */
export function stringifyItem(
  item: unknown,
  indent: string,
  depth = 0,
): string {
  if (item == null) return "null";
  if (typeof item !== "object") {
    return typeof item === "string" ? JSON.stringify(item) : String(item);
  }
  if (Array.isArray(item)) {
    if (item.length === 0) return "[]";
    if (depth >= MAX_STRINGIFY_DEPTH) return `Array(${item.length})`;
    const shown = item.slice(0, MAX_ARRAY_ITEMS);
    const inner = shown
      .map((el) => `${indent}  ${stringifyItem(el, indent + "  ", depth + 1)}`)
      .join(",\n");
    const remaining = item.length - MAX_ARRAY_ITEMS;
    const suffix = remaining > 0 ? `,\n${indent}  ... (${remaining} more)` : "";
    return `[\n${inner}${suffix}\n${indent}]`;
  }
  const entries = Object.entries(item);
  if (entries.length === 0) return "{}";
  if (depth >= MAX_STRINGIFY_DEPTH) return `Object(${entries.length} keys)`;
  const inner = entries
    .map(
      ([k, v]) =>
        `${indent}  ${k}: ${stringifyItem(v, indent + "  ", depth + 1)}`,
    )
    .join(",\n");
  return `{\n${inner}\n${indent}}`;
}

/** Build a lightweight summary string for a large value without JSON.stringify. */
export function summarizeValue(value: unknown): string {
  const resolved = resolveValue(value);
  if (Array.isArray(resolved)) {
    const len = resolved.length;
    if (len === 0) return "[] (empty array)";
    // Show first item fully expanded so the user sees the complete schema
    const preview = resolved
      .slice(0, ARRAY_PREVIEW_ITEMS)
      .map((item) => stringifyItem(item, "  "))
      .join(",\n  ");
    const remaining = len - ARRAY_PREVIEW_ITEMS;
    const suffix = remaining > 0 ? `,\n  ... (${remaining} more items)` : "";
    return `Array(${len}) [\n  ${preview}${suffix}\n]`;
  }
  if (typeof resolved === "object" && resolved !== null) {
    const entries = Object.entries(resolved);
    if (entries.length === 0) return "{} (empty object)";
    const preview = entries
      .slice(0, OBJECT_PREVIEW_KEYS)
      .map(([k, v]) => `  ${k}: ${stringifyItem(v, "  ")}`)
      .join(",\n");
    const remaining = entries.length - OBJECT_PREVIEW_KEYS;
    const suffix = remaining > 0 ? `,\n  ... (${remaining} more keys)` : "";
    return `Object(${entries.length} keys) {\n${preview}${suffix}\n}`;
  }
  return String(resolved);
}

/** A string safe to run a substring search over, whatever the value's size. */
export function toSearchableString(value: unknown): string {
  if (typeof value !== "object" || value === null) return String(value);
  if (isLargeValue(value)) return summarizeValue(value);
  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
}

/**
 * Serialize a value for a table cell's accessor. Small values are serialized
 * whole so a global filter can match anywhere inside them; large ones fall
 * back to a summary, which is the only thing that keeps a file full of meshes
 * or embedded textures from stalling the table.
 */
export function safeSerialize(value: unknown): string {
  if (value === undefined) return "-";
  if (value === null) return "null";

  if (typeof value === "object" && isLargeValue(value)) {
    return summarizeValue(value);
  }

  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
}
