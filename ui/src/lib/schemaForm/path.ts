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
 * It does assume no property name contains a `.`, which holds for every one of
 * the 369 names in the action corpus (serde renames them all to camelCase) and
 * is checked by `path.test.ts`. What this replaces — reconstructing a path by
 * splitting RJSF's DOM id on `_` — made the same bet on underscores, silently.
 */
import type { FieldPath } from "./types";

/** Dot path for a field. The root is `""`. */
export const pathKey = (path: FieldPath): string => path.join(".");

/**
 * Inverse of `pathKey`. Segments that look like array indices come back as
 * numbers, so `setAtPath` rebuilds an array rather than an object — which
 * `path.split(".")` alone did not.
 */
export const parsePathKey = (key: string): FieldPath => {
  if (key === "") return [];
  return key
    .split(".")
    .map((segment) =>
      /^(0|[1-9]\d*)$/.test(segment) ? Number(segment) : segment,
    );
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

/** Immutably remove the value at `path`. Used to clear a nullable field. */
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
