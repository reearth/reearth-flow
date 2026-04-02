import { setValueAtPath } from "./fieldUtils";

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
      result = setValueAtPath(result, path.split("."), value);
    });

  return result;
};

export const rjsfIdToPath = (changedFieldId?: string) => {
  if (!changedFieldId) return undefined;
  return changedFieldId
    .replace(/__anyof_select$/, "")
    .replace(/__oneof_select$/, "")
    .replace(/^root_/, "")
    .replace(/_/g, ".");
};
