/**
 * Describing arbitrary feature values without serializing all of them.
 *
 * Intermediate-data features carry values with no useful upper bound — mesh
 * face lists, point-cloud segments, per-corner UV. Anything that reaches for
 * `JSON.stringify` on every value it is handed will eventually be handed one of
 * those, so the entry points here measure first and print under a budget when
 * measuring says to.
 *
 * Two printers, because the two places that show a value want opposite things:
 * {@link previewSerialize} produces one compact line for a table cell, and
 * {@link formatStructured} produces an indented block for the details panel.
 * Both keep a coordinate — an array of numbers — on a single line, since the
 * point of looking at geometry is reading positions, and a printer that breaks
 * `[139.7, 35.6, 10]` across three lines buries them.
 */

/** Leaf count past which a value is previewed rather than serialized whole. */
export const LARGE_VALUE_THRESHOLD = 100;

/** Nesting depth past which either printer reports a shape instead of descending. */
const MAX_DEPTH = 12;

/** Characters {@link previewSerialize} may emit — a cell's accessor string. */
const PREVIEW_BUDGET = 400;

/** Characters {@link formatStructured} may emit — a scrollable detail block. */
const BLOCK_BUDGET = 20_000;

/** Numbers shown before an inlined coordinate list is elided. */
const INLINE_NUMBERS = 12;

/** Array items, or object keys, shown per level by {@link formatStructured}. */
const BLOCK_ENTRIES = 40;

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

/** A single coordinate, which reads as a line rather than as a column. */
function isNumberArray(value: unknown[]): boolean {
  return value.length > 0 && value.every((item) => typeof item === "number");
}

function scalarText(value: unknown): string {
  if (value === null) return "null";
  if (value === undefined) return "undefined";
  if (typeof value === "string") return JSON.stringify(value);
  return String(value);
}

/** How the printers say "there was more here", with the count that was cut. */
function elided(count: number): string {
  return `… ${count.toLocaleString()} more`;
}

/** Shape note for a value the printer stopped short of, at its depth limit. */
function shapeText(value: object): string {
  return Array.isArray(value)
    ? `Array(${value.length})`
    : `Object(${Object.keys(value).length} keys)`;
}

/** Remaining characters a printer may emit. Also what bounds a cyclic value. */
type Budget = { left: number };

function writeCompact(value: unknown, budget: Budget, depth: number): string {
  if (value === null || typeof value !== "object") {
    const text = scalarText(value);
    budget.left -= text.length;
    return text;
  }

  budget.left -= 2; // The brackets, which also guarantees a cycle runs out.
  if (depth >= MAX_DEPTH) return shapeText(value);

  if (Array.isArray(value)) {
    if (value.length === 0) return "[]";

    // A single coordinate goes out whole even if that overruns the budget:
    // stopping halfway through one reads as a wrong position, not a cut-off
    // list, and the overrun is a few characters.
    if (isNumberArray(value) && value.length <= INLINE_NUMBERS) {
      const text = `[${value.join(", ")}]`;
      budget.left -= text.length;
      return text;
    }

    const parts: string[] = [];
    for (const item of value) {
      if (budget.left <= 0) break;
      if (parts.length > 0) budget.left -= 2; // ", "
      parts.push(writeCompact(item, budget, depth + 1));
    }
    if (parts.length < value.length) {
      parts.push(elided(value.length - parts.length));
    }
    return `[${parts.join(", ")}]`;
  }

  const entries = Object.entries(value as Record<string, unknown>);
  if (entries.length === 0) return "{}";

  const parts: string[] = [];
  for (const [key, item] of entries) {
    if (budget.left <= 0) break;
    budget.left -= key.length + (parts.length > 0 ? 4 : 2); // `, key: `
    parts.push(`${key}: ${writeCompact(item, budget, depth + 1)}`);
  }
  if (parts.length < entries.length) {
    parts.push(elided(entries.length - parts.length));
  }
  return `{${parts.join(", ")}}`;
}

/**
 * One compact line describing a value, cut off once it has spent its budget.
 *
 * Emits real JSON-ish content — numbers as numbers — for as far as the budget
 * reaches, rather than a count and a shape. A table cell shows the first ~100
 * characters of this, and for geometry those characters should be coordinates.
 */
export function previewSerialize(
  value: unknown,
  budget: number = PREVIEW_BUDGET,
): string {
  return writeCompact(value, { left: budget }, 0);
}

function writeBlock(
  value: unknown,
  budget: Budget,
  indent: string,
  depth: number,
): string {
  if (value === null || typeof value !== "object") {
    const text = scalarText(value);
    budget.left -= text.length;
    return text;
  }

  budget.left -= 2;
  if (depth >= MAX_DEPTH) return shapeText(value);

  if (Array.isArray(value)) {
    if (value.length === 0) return "[]";

    // A coordinate stays on one line; everything else opens a block.
    if (isNumberArray(value)) {
      const shown = value.slice(0, INLINE_NUMBERS);
      const rest = value.length - shown.length;
      const text = `[${shown.join(", ")}${rest > 0 ? `, ${elided(rest)}` : ""}]`;
      budget.left -= text.length;
      return text;
    }

    const lines: string[] = [];
    for (const item of value) {
      if (budget.left <= 0 || lines.length >= BLOCK_ENTRIES) break;
      budget.left -= indent.length + 3; // The indent, and ",\n"
      lines.push(
        `${indent}  ${writeBlock(item, budget, indent + "  ", depth + 1)}`,
      );
    }
    const rest = value.length - lines.length;
    if (rest > 0) lines.push(`${indent}  ${elided(rest)}`);
    return `[\n${lines.join(",\n")}\n${indent}]`;
  }

  const entries = Object.entries(value as Record<string, unknown>);
  if (entries.length === 0) return "{}";

  const lines: string[] = [];
  for (const [key, item] of entries) {
    if (budget.left <= 0 || lines.length >= BLOCK_ENTRIES) break;
    budget.left -= indent.length + key.length + 5; // The indent, `key: ` and ",\n"
    lines.push(
      `${indent}  ${key}: ${writeBlock(item, budget, indent + "  ", depth + 1)}`,
    );
  }
  const rest = entries.length - lines.length;
  if (rest > 0) lines.push(`${indent}  ${elided(rest)}`);
  return `{\n${lines.join(",\n")}\n${indent}}`;
}

/**
 * An indented block describing a value, cut off once it has spent its budget.
 *
 * This is what the details panel renders, for values of any size — the budget
 * is what makes a 300-face mesh safe to hand it, so there is no separate path
 * for large values to fall down. The full value stays reachable through the
 * raw viewer.
 */
export function formatStructured(
  value: unknown,
  budget: number = BLOCK_BUDGET,
): string {
  return writeBlock(value, { left: budget }, "", 0);
}

/** A string safe to run a substring search over, whatever the value's size. */
export function toSearchableString(value: unknown): string {
  if (typeof value !== "object" || value === null) return String(value);
  if (isLargeValue(value)) return previewSerialize(value);
  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
}

/**
 * Serialize a value for a table cell's accessor. Small values are serialized
 * whole so a global filter can match anywhere inside them; large ones fall
 * back to a bounded preview, which is the only thing that keeps a file full of
 * meshes or embedded textures from stalling the table.
 */
export function safeSerialize(value: unknown): string {
  if (value === undefined) return "-";
  if (value === null) return "null";

  if (typeof value === "object" && isLargeValue(value)) {
    return previewSerialize(value);
  }

  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
}
