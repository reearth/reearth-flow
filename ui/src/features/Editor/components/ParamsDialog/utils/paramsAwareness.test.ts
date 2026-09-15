/**
 * Draft patches are shared over Yjs, so the key format here is a wire format.
 * These pin the two things that move through it: that a patch lands where it
 * says it does, and that the root path is a real path rather than "no path".
 */
import { describe, expect, it } from "vitest";

import {
  applyMergedPatch,
  changedFieldPath,
  diffToPatch,
  type DraftPatch,
} from "./paramsAwareness";

const drafts = (paramsPatch: DraftPatch["paramsPatch"]) => ({
  someClient: { paramsPatch },
});

describe("changedFieldPath", () => {
  it("passes a dot path straight through", () => {
    expect(changedFieldPath("response.responseEncoding")).toBe(
      "response.responseEncoding",
    );
  });

  it("keeps the root path, which a union-rooted schema writes to", () => {
    // Feature Reader and the other three union-rooted actions change their
    // whole params object at `""`; treating that as falsy dropped the edit.
    expect(changedFieldPath("")).toBe("");
  });

  it("reports no path when the form named no field", () => {
    expect(changedFieldPath(undefined)).toBeUndefined();
  });
});

describe("applyMergedPatch", () => {
  it("applies a nested patch over the saved params", () => {
    const result = applyMergedPatch(
      { response: { responseBodyAttribute: "_body" } },
      drafts({
        "response.responseEncoding": { value: "base64", updatedAt: 1 },
      }),
      "paramsPatch",
    );
    expect(result).toEqual({
      response: { responseBodyAttribute: "_body", responseEncoding: "base64" },
    });
  });

  it("replaces the whole object for a root patch", () => {
    expect(
      applyMergedPatch(
        { type: "csv" },
        drafts({ "": { value: { type: "json" }, updatedAt: 1 } }),
        "paramsPatch",
      ),
    ).toEqual({ type: "json" });
  });

  it("keeps an array an array when patching into one", () => {
    const result = applyMergedPatch(
      { rules: [{ name: "a" }, { name: "b" }] },
      drafts({ "rules.1.name": { value: "c", updatedAt: 1 } }),
      "paramsPatch",
    );
    expect(Array.isArray((result as { rules: unknown }).rules)).toBe(true);
    expect(result).toEqual({ rules: [{ name: "a" }, { name: "c" }] });
  });

  it("applies patches in the order they were made", () => {
    expect(
      applyMergedPatch(
        {},
        {
          first: { paramsPatch: { name: { value: "early", updatedAt: 1 } } },
          second: { paramsPatch: { name: { value: "late", updatedAt: 2 } } },
        },
        "paramsPatch",
      ),
    ).toEqual({ name: "late" });
  });

  it("returns a copy of the base when there are no drafts", () => {
    const base = { a: 1 };
    const result = applyMergedPatch(base, undefined, "paramsPatch");
    expect(result).toEqual(base);
    expect(result).not.toBe(base);
  });
});

describe("diffToPatch", () => {
  it("records only what changed", () => {
    const patch = diffToPatch(
      { a: 1, nested: { b: 2 } },
      { a: 1, nested: { b: 3 } },
    );
    expect(Object.keys(patch)).toEqual(["nested.b"]);
    expect(patch["nested.b"].value).toBe(3);
  });
});
