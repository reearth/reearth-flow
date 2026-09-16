/**
 * Draft patches are shared over Yjs, so the key format here is a wire format.
 * These pin the two things that move through it: that a patch lands where it
 * says it does, and that the root path is a real path rather than "no path".
 */
import { describe, expect, it } from "vitest";

import { parsePathKey } from "@flow/lib/schemaForm";

import {
  applyMergedPatch,
  changedFieldPath,
  diffToPatch,
  nextSeq,
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

describe("ordering between clients", () => {
  // Wall-clock time cannot order edits across machines. These pin the causal
  // ordering that replaced it.
  const at = (seq: number, updatedAt: number, value: unknown) => ({
    value,
    updatedAt,
    seq,
  });

  it("ignores a fast clock in favour of what each client had seen", () => {
    const merged = applyMergedPatch(
      {},
      {
        // This client's clock runs an hour fast, but it wrote first.
        fastClock: { paramsPatch: { name: at(1, 3_600_000, "older") } },
        // This one saw that edit and wrote after it, with a correct clock.
        goodClock: { paramsPatch: { name: at(2, 1_000, "newer") } },
      },
      "paramsPatch",
    );
    expect(merged).toEqual({ name: "newer" });
  });

  it("breaks a genuine tie the same way on every replica", () => {
    const drafts = {
      bbb: { paramsPatch: { name: at(4, 10, "from-b") } },
      aaa: { paramsPatch: { name: at(4, 10, "from-a") } },
    };
    const forwards = applyMergedPatch({}, drafts, "paramsPatch");
    const backwards = applyMergedPatch(
      {},
      Object.fromEntries(Object.entries(drafts).reverse()),
      "paramsPatch",
    );
    // Whichever order the drafts happen to be enumerated in, both clients
    // converge on the same value.
    expect(forwards).toEqual(backwards);
    expect(forwards).toEqual({ name: "from-b" });
  });

  it("still orders drafts written without a counter", () => {
    const merged = applyMergedPatch(
      {},
      {
        legacyA: { paramsPatch: { name: { value: "first", updatedAt: 1 } } },
        legacyB: { paramsPatch: { name: { value: "second", updatedAt: 2 } } },
      },
      "paramsPatch",
    );
    expect(merged).toEqual({ name: "second" });
  });

  it("puts a counted edit after an uncounted one", () => {
    const merged = applyMergedPatch(
      {},
      {
        legacy: { paramsPatch: { name: { value: "old", updatedAt: 9_999 } } },
        current: { paramsPatch: { name: at(1, 1, "new") } },
      },
      "paramsPatch",
    );
    expect(merged).toEqual({ name: "new" });
  });
});

/**
 * Whole sessions rather than single comparisons.
 *
 * The merge is the convergence point of the collaborative layer: every client
 * holds the same set of drafts and has to arrive at the same params from them,
 * whatever order Yjs happens to hand them over in. The tests above pin one
 * decision at a time; these pin the property that matters — that two people
 * editing at once end up looking at the same thing, with nobody's work gone.
 */
describe("concurrent sessions", () => {
  type Entry = { value: unknown; updatedAt: number; seq: number };
  type Store = Record<string, { paramsPatch: Record<string, Entry> }>;

  /** Every ordering of the clients, which is what differs between replicas. */
  const permutations = <T>(items: T[]): T[][] =>
    items.length <= 1
      ? [items]
      : items.flatMap((item, index) =>
          permutations([
            ...items.slice(0, index),
            ...items.slice(index + 1),
          ]).map((rest) => [item, ...rest]),
        );

  const replicas = (base: unknown, store: Store) =>
    permutations(Object.keys(store)).map((order) =>
      applyMergedPatch(
        base,
        Object.fromEntries(order.map((client) => [client, store[client]])),
        "paramsPatch",
      ),
    );

  it("converges on one result however the drafts are enumerated", () => {
    // Three people in the same node: two contesting `name`, everyone adding
    // fields of their own, one clearing a field that was already saved.
    const store: Store = {
      alice: {
        paramsPatch: {
          name: { value: "from-alice", updatedAt: 100, seq: 3 },
          "rules.0.attribute": { value: "bldg", updatedAt: 101, seq: 4 },
        },
      },
      bob: {
        paramsPatch: {
          name: { value: "from-bob", updatedAt: 90, seq: 5 },
          threshold: { value: 12, updatedAt: 95, seq: 2 },
        },
      },
      carol: {
        paramsPatch: {
          encoding: { value: undefined, updatedAt: 80, seq: 1 },
        },
      },
    };

    const results = replicas({ encoding: "utf-8", keep: true }, store);

    expect(results).toHaveLength(6);
    for (const result of results) expect(result).toEqual(results[0]);

    expect(results[0]).toEqual({
      keep: true,
      // Bob's counter is higher, so his edit is the one that saw Alice's.
      name: "from-bob",
      threshold: 12,
      rules: [{ attribute: "bldg" }],
    });
    // Cleared, not set to undefined — the key is gone.
    expect(results[0]).not.toHaveProperty("encoding");
  });

  it("loses nobody's edit when they touch different fields", () => {
    const store: Store = Object.fromEntries(
      ["alice", "bob", "carol", "dave"].map((client, index) => [
        client,
        {
          paramsPatch: {
            [`field${index}`]: { value: client, updatedAt: index, seq: 1 },
          },
        },
      ]),
    );

    for (const result of replicas({}, store)) {
      expect(result).toEqual({
        field0: "alice",
        field1: "bob",
        field2: "carol",
        field3: "dave",
      });
    }
  });

  it("converges when two clients edit at once and then see each other", () => {
    // The counter is taken from what the writer has seen, so this walks a real
    // session: a concurrent pair, then an edit made after the exchange.
    const alice: Store["x"]["paramsPatch"] = {};
    const bob: Store["x"]["paramsPatch"] = {};

    // Neither has seen the other — both count from an empty node.
    alice.name = { value: "alice-1", updatedAt: 10, seq: nextSeq({}) };
    bob.threshold = { value: 1, updatedAt: 11, seq: nextSeq({}) };
    expect(alice.name.seq).toBe(bob.threshold.seq);

    // They exchange, and Alice edits again knowing about both.
    const seen: Store = {
      alice: { paramsPatch: alice },
      bob: { paramsPatch: bob },
    };
    alice.name = { value: "alice-2", updatedAt: 12, seq: nextSeq(seen) };
    expect(alice.name.seq).toBeGreaterThan(bob.threshold.seq);

    for (const result of replicas({}, seen)) {
      expect(result).toEqual({ name: "alice-2", threshold: 1 });
    }
  });

  it("orders a clear against a set by the counter, not by the clock", () => {
    const cleared: Store = {
      writer: {
        paramsPatch: { name: { value: "typed", updatedAt: 50, seq: 1 } },
      },
      clearer: {
        paramsPatch: { name: { value: undefined, updatedAt: 40, seq: 2 } },
      },
    };
    for (const result of replicas({ name: "saved" }, cleared)) {
      expect(result).not.toHaveProperty("name");
    }

    // The same pair the other way round: the clear happened first, so the
    // value typed after it survives.
    const retyped: Store = {
      clearer: {
        paramsPatch: { name: { value: undefined, updatedAt: 40, seq: 1 } },
      },
      writer: {
        paramsPatch: { name: { value: "typed", updatedAt: 50, seq: 2 } },
      },
    };
    for (const result of replicas({ name: "saved" }, retyped)) {
      expect(result).toEqual({ name: "typed" });
    }
  });

  it("does not let one client's patch corrupt another's container", () => {
    // A map key with a dot and an array index under the same parent: the two
    // spellings the key format exists to keep apart.
    const store: Store = {
      alice: {
        paramsPatch: {
          "inline.bldg~1Building": { value: "a", updatedAt: 1, seq: 1 },
        },
      },
      bob: {
        paramsPatch: { "rules.0.name": { value: "b", updatedAt: 2, seq: 2 } },
      },
    };

    for (const result of replicas({}, store)) {
      expect(result).toEqual({
        inline: { "bldg.Building": "a" },
        rules: [{ name: "b" }],
      });
    }
  });
});

describe("nextSeq", () => {
  it("starts at one for a node nobody has edited", () => {
    expect(nextSeq(undefined)).toBe(1);
    expect(nextSeq({})).toBe(1);
  });

  it("rises above every counter already present, across clients and tabs", () => {
    expect(
      nextSeq({
        a: {
          paramsPatch: { x: { value: 1, updatedAt: 0, seq: 3 } },
          customizationsPatch: { y: { value: 1, updatedAt: 0, seq: 7 } },
        },
        b: { paramsPatch: { z: { value: 1, updatedAt: 0, seq: 5 } } },
      }),
    ).toBe(8);
  });

  it("treats a draft with no counter as zero", () => {
    expect(
      nextSeq({ a: { paramsPatch: { x: { value: 1, updatedAt: 500 } } } }),
    ).toBe(1);
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

  it("keys a dotted map entry the way a patch is applied", () => {
    // Flattening joined on "." directly, which produced a key that reads back
    // as two segments and would patch a different object than it named.
    const patch = diffToPatch(
      { inline: { "bldg.part": 1 } },
      { inline: { "bldg.part": 2 } },
    );
    const [key] = Object.keys(patch);
    expect(parsePathKey(key)).toEqual(["inline", "bldg.part"]);

    const merged = applyMergedPatch(
      { inline: { "bldg.part": 1 } },
      drafts(patch),
      "paramsPatch",
    );
    expect(merged).toEqual({ inline: { "bldg.part": 2 } });
  });
});
