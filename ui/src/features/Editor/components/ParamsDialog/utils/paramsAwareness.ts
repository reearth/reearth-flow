import { parsePathKey, setAtPath } from "@flow/lib/schemaForm";

type PatchEntry = {
  value: any;
  updatedAt: number;
};

export type DraftPatch = {
  paramsPatch?: Record<string, PatchEntry>;
  customizationsPatch?: Record<string, PatchEntry>;
};

type NodeDrafts = Record<string, DraftPatch | undefined>;
export type DraftStore = Record<string, NodeDrafts | undefined>;

export const flattenObject = (
  obj: any,
  prefix = "",
  result: Record<string, any> = {},
): Record<string, any> => {
  if (obj === null || obj === undefined) return result;

  if (typeof obj !== "object" || Array.isArray(obj)) {
    if (prefix) result[prefix] = obj;
    return result;
  }

  Object.entries(obj).forEach(([key, value]) => {
    const path = prefix ? `${prefix}.${key}` : key;

    if (value !== null && typeof value === "object" && !Array.isArray(value)) {
      flattenObject(value, path, result);
    } else {
      result[path] = value;
    }
  });

  return result;
};

export const diffToPatch = (
  base: any,
  next: any,
): Record<string, PatchEntry> => {
  const baseFlat = flattenObject(base ?? {});
  const nextFlat = flattenObject(next ?? {});
  const allPaths = new Set([
    ...Object.keys(baseFlat),
    ...Object.keys(nextFlat),
  ]);

  const now = Date.now();
  const patch: Record<string, PatchEntry> = {};

  allPaths.forEach((path) => {
    const baseValue = baseFlat[path];
    const nextValue = nextFlat[path];

    if (JSON.stringify(baseValue) !== JSON.stringify(nextValue)) {
      patch[path] = {
        value: nextValue,
        updatedAt: now,
      };
    }
  });

  return patch;
};

export const applyMergedPatch = (
  base: any,
  drafts: NodeDrafts | undefined,
  patchKey: "paramsPatch" | "customizationsPatch",
) => {
  let result = structuredClone(base ?? {});
  if (!drafts) return result;

  const allEntries: { path: string; value: any; updatedAt: number }[] = [];

  Object.values(drafts).forEach((draft) => {
    const patch = draft?.[patchKey];
    if (!patch) return;

    Object.entries(patch).forEach(([path, entry]) => {
      allEntries.push({
        path,
        value: entry.value,
        updatedAt: entry.updatedAt,
      });
    });
  });

  allEntries
    .sort((a, b) => a.updatedAt - b.updatedAt)
    .forEach(({ path, value }) => {
      // `parsePathKey` reads a numeric segment back as a number, so a patch
      // into `rules.0.name` rebuilds an array rather than an object keyed "0".
      result = setAtPath(result, parsePathKey(path), value);
    });

  return result;
};

/**
 * The form reports the dot path of the field it changed, which is already the
 * key a draft patch is filed under, so there is nothing to translate.
 *
 * The empty string is a real path — the form's root — and must be kept. Four
 * actions (Feature Reader, JSON/XML Fragmenter, PLATEAU4.SolarPositionCalculator)
 * have a union rather than an object at the root of their schema, so choosing a
 * variant on them changes the whole params object at path `""`. Rejecting it as
 * falsy dropped the edit silently.
 */
export const changedFieldPath = (
  changedFieldKey?: string,
): string | undefined => changedFieldKey;
