import {
  deleteAtPath,
  parsePathKey,
  pathKey,
  setAtPath,
} from "@flow/lib/schemaForm";

type PatchEntry = {
  value: any;
  updatedAt: number;
  /**
   * Lamport counter: one more than the highest any client has written to this
   * node's drafts. Wall-clock time cannot order edits between clients — a
   * client whose clock runs fast makes its older edit sort after a newer one
   * and overwrite it, and two edits in the same millisecond have no order at
   * all. `updatedAt` is kept for display and for drafts written by a client
   * that predates this field.
   */
  seq?: number;
};

export type DraftPatch = {
  paramsPatch?: Record<string, PatchEntry>;
  customizationsPatch?: Record<string, PatchEntry>;
};

/** Every client's draft for one node, keyed by client id. */
export type NodeDrafts = Record<string, DraftPatch | undefined>;
export type DraftStore = Record<string, NodeDrafts | undefined>;

/**
 * The counter to stamp on the next edit: above every counter already present in
 * this node's drafts, so an edit made after seeing another client's edit sorts
 * after it regardless of either clock.
 */
export const nextSeq = (drafts: NodeDrafts | undefined): number => {
  let highest = 0;
  for (const draft of Object.values(drafts ?? {})) {
    for (const patchKey of ["paramsPatch", "customizationsPatch"] as const) {
      for (const entry of Object.values(draft?.[patchKey] ?? {})) {
        if (typeof entry.seq === "number" && entry.seq > highest) {
          highest = entry.seq;
        }
      }
    }
  }
  return highest + 1;
};

/**
 * Flattens a value to one entry per leaf, keyed the same way a field is.
 *
 * Keys go through `pathKey` rather than being joined on `.` directly: a map
 * entry the user named `bldg.part` would otherwise produce a key that reads
 * back as two segments and patches a different object.
 */
export const flattenObject = (
  obj: any,
  prefix: (string | number)[] = [],
  result: Record<string, any> = {},
): Record<string, any> => {
  if (obj === null || obj === undefined) return result;

  if (typeof obj !== "object" || Array.isArray(obj)) {
    if (prefix.length > 0) result[pathKey(prefix)] = obj;
    return result;
  }

  Object.entries(obj).forEach(([key, value]) => {
    const path = [...prefix, key];

    if (value !== null && typeof value === "object" && !Array.isArray(value)) {
      flattenObject(value, path, result);
    } else {
      result[pathKey(path)] = value;
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

  const allEntries: {
    path: string;
    value: any;
    updatedAt: number;
    seq: number;
    clientId: string;
  }[] = [];

  Object.entries(drafts).forEach(([clientId, draft]) => {
    const patch = draft?.[patchKey];
    if (!patch) return;

    Object.entries(patch).forEach(([path, entry]) => {
      allEntries.push({
        path,
        value: entry.value,
        updatedAt: entry.updatedAt,
        seq: entry.seq ?? 0,
        clientId,
      });
    });
  });

  allEntries
    // Causal order first. `updatedAt` then separates drafts from a client that
    // writes no counter, and the client id breaks a genuine tie the same way on
    // every replica, so all of them merge to the same result.
    .sort(
      (a, b) =>
        a.seq - b.seq ||
        a.updatedAt - b.updatedAt ||
        (a.clientId < b.clientId ? -1 : a.clientId > b.clientId ? 1 : 0),
    )
    .forEach(({ path, value }) => {
      // `parsePathKey` reads a numeric segment back as a number, so a patch
      // into `rules.0.name` rebuilds an array rather than an object keyed "0".
      const segments = parsePathKey(path);
      // A cleared field patches to `undefined`, and Yjs drops the key on the
      // way out — so a collaborator reads the same absence either way. Remove
      // the key rather than writing `undefined` back over it.
      result =
        value === undefined
          ? deleteAtPath(result, segments)
          : setAtPath(result, segments, value);
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
