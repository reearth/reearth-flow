/**
 * Field identity.
 *
 * Paths are structured — `["response", "responseEncoding"]`, `["rules", 0,
 * "attribute"]` — and serialise to a dot path (`rules.0.attribute`) for use as
 * a map key: awareness focus, error lookup, and the draft patches shared over
 * Yjs.
 *
 * Dot rather than JSON Pointer because the draft patches in `paramDrafts`
 * already use this spelling. Keeping it means the collaborative wire format is
 * untouched by the move off RJSF, so clients on either build agree about which
 * field a patch belongs to.
 *
 * Two things in a segment would otherwise be read back as something else, so
 * both are escaped:
 *
 * - a `.` inside a name would split into two segments. Schema property names
 *   never contain one (`path.test.ts` checks that against the whole corpus),
 *   but the keys of a `map` field are typed by the user, and a key like
 *   `roads.highway` was being read and patched as `map.roads.highway` — a
 *   nested pair of fields rather than the one entry, corrupting the edit and
 *   every collaborator's copy of it.
 * - a name made only of digits would be read back as an array index, so a map
 *   entry called `0` would rebuild the map as an array.
 *
 * The escapes follow JSON Pointer's convention: `~0` for a literal `~`, `~1`
 * for a literal `.`, plus `~2` prefixing a string that would otherwise read as
 * an index. Array indices stay bare digits, which is what keeps the existing
 * wire format intact.
 */
import type { FieldPath } from "./types";

const INDEX = /^(0|[1-9]\d*)$/;

/** True when this string would be read back as an array index. */
const readsAsIndex = (segment: string): boolean => INDEX.test(segment);

const escapeSegment = (segment: string): string => {
  // `~` first, so the markers introduced below are never re-escaped.
  const escaped = segment.replace(/~/g, "~0").replace(/\./g, "~1");
  return readsAsIndex(escaped) ? `~2${escaped}` : escaped;
};

const unescapeSegment = (segment: string): string =>
  segment.replace(/~1/g, ".").replace(/~0/g, "~");

/** Dot path for a field. The root is `""`. */
export const pathKey = (path: FieldPath): string =>
  path
    .map((segment) =>
      typeof segment === "number" ? String(segment) : escapeSegment(segment),
    )
    .join(".");

/**
 * Inverse of `pathKey`. A bare run of digits comes back as a number, so
 * `setAtPath` rebuilds an array rather than an object keyed `"0"`.
 */
export const parsePathKey = (key: string): FieldPath => {
  if (key === "") return [];
  return key.split(".").map((segment) => {
    if (segment.startsWith("~2")) return unescapeSegment(segment.slice(2));
    if (readsAsIndex(segment)) return Number(segment);
    return unescapeSegment(segment);
  });
};

export const childPath = (
  path: FieldPath,
  segment: string | number,
): FieldPath => [...path, segment];

export const getAtPath = (value: unknown, path: FieldPath): unknown =>
  path.reduce<unknown>(
    (current, segment) =>
      current === null || current === undefined
        ? undefined
        : (current as Record<string | number, unknown>)[segment],
    value,
  );

/**
 * Immutably set `value` at `path`, creating containers as needed. A numeric
 * segment creates an array, a string segment an object, so a path never
 * silently turns one into the other.
 */
export const setAtPath = (
  target: unknown,
  path: FieldPath,
  value: unknown,
): unknown => {
  if (path.length === 0) return value;

  const [segment, ...rest] = path;
  const wantsArray = typeof segment === "number";

  const container: any = Array.isArray(target)
    ? [...target]
    : target && typeof target === "object"
      ? { ...(target as object) }
      : wantsArray
        ? []
        : {};

  container[segment] = setAtPath(container[segment], rest, value);
  return container;
};

/**
 * Immutably remove the value at `path`.
 *
 * Clearing a field removes its key rather than setting it to `undefined`. A
 * key that is present with an undefined value is still a key: `Object.keys`
 * reports it, so a schema with `additionalProperties: false` rejects the object
 * it sits in, and it survives into the params as a phantom entry.
 */
export const deleteAtPath = (target: unknown, path: FieldPath): unknown => {
  if (path.length === 0) return undefined;

  const [segment, ...rest] = path;
  if (!target || typeof target !== "object") return target;

  if (rest.length === 0) {
    if (Array.isArray(target)) {
      if (typeof segment !== "number") return target;
      return target.filter((_, index) => index !== segment);
    }
    const key = String(segment);
    return Object.fromEntries(
      Object.entries(target as Record<string, unknown>).filter(
        ([entry]) => entry !== key,
      ),
    );
  }

  const container: any = Array.isArray(target)
    ? [...target]
    : { ...(target as object) };
  container[segment] = deleteAtPath(container[segment], rest);
  return container;
};
