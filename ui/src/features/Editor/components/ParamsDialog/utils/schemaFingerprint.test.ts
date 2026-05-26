import { RJSFSchema } from "@rjsf/utils";
import { describe, it, expect } from "vitest";

import {
  computeSchemaFingerprint,
  schemaKeysMatch,
} from "./schemaFingerprint";

const s = (properties: Record<string, object>): RJSFSchema =>
  ({ properties } as RJSFSchema);

describe("computeSchemaFingerprint", () => {
  it("returns undefined when schema is undefined", () => {
    expect(computeSchemaFingerprint(undefined)).toBeUndefined();
  });

  it("returns undefined when schema has no properties", () => {
    expect(computeSchemaFingerprint({ type: "object" })).toBeUndefined();
  });

  it("returns a string for a valid schema", () => {
    expect(typeof computeSchemaFingerprint(s({ a: { type: "string" } }))).toBe(
      "string",
    );
  });

  it("is invariant to property definition order (same keys, same fingerprint)", () => {
    const a = s({ alpha: { type: "string" }, beta: { type: "number" } });
    const b = s({ beta: { type: "number" }, alpha: { type: "string" } });
    expect(computeSchemaFingerprint(a)).toBe(computeSchemaFingerprint(b));
  });

  it("produces different fingerprints when keys differ", () => {
    const a = s({ a: { type: "string" } });
    const b = s({ b: { type: "string" } });
    expect(computeSchemaFingerprint(a)).not.toBe(computeSchemaFingerprint(b));
  });

  it("produces the same fingerprint when only values change (type)", () => {
    const a = s({ a: { type: "string" } });
    const b = s({ a: { type: "number" } });
    expect(computeSchemaFingerprint(a)).toBe(computeSchemaFingerprint(b));
  });

  it("produces the same fingerprint when only values change (description)", () => {
    const a = s({ a: { type: "string", description: "old" } });
    const b = s({ a: { type: "string", description: "new" } });
    expect(computeSchemaFingerprint(a)).toBe(computeSchemaFingerprint(b));
  });

  it("produces the same fingerprint when only values change (enum values)", () => {
    const a = s({ mode: { type: "string", enum: ["a", "b"] } });
    const b = s({ mode: { type: "string", enum: ["a", "b", "c"] } });
    expect(computeSchemaFingerprint(a)).toBe(computeSchemaFingerprint(b));
  });
});

describe("schemaKeysMatch", () => {
  it("returns true when storedSchema is undefined (first-time node)", () => {
    expect(schemaKeysMatch(undefined, s({ a: { type: "string" } }))).toBe(true);
  });

  it("returns true when keys are identical", () => {
    const stored = s({ a: { type: "string" }, b: { type: "number" } });
    const current = s({ a: { type: "string" }, b: { type: "number" } });
    expect(schemaKeysMatch(stored, current)).toBe(true);
  });

  it("returns true when key order differs between stored and current", () => {
    const stored = s({ a: { type: "string" }, b: { type: "number" } });
    const current = s({ b: { type: "number" }, a: { type: "string" } });
    expect(schemaKeysMatch(stored, current)).toBe(true);
  });

  it("returns true when property values change but keys are the same", () => {
    const stored = s({ url: { type: "string" }, count: { type: "string" } });
    const current = s({
      url: { type: "string", description: "The endpoint URL" },
      count: { type: "number", default: 10 },
    });
    expect(schemaKeysMatch(stored, current)).toBe(true);
  });

  it("returns false when a property is added", () => {
    const stored = s({ a: { type: "string" } });
    const current = s({ a: { type: "string" }, b: { type: "number" } });
    expect(schemaKeysMatch(stored, current)).toBe(false);
  });

  it("returns false when a property is removed", () => {
    const stored = s({ a: { type: "string" }, b: { type: "number" } });
    const current = s({ a: { type: "string" } });
    expect(schemaKeysMatch(stored, current)).toBe(false);
  });

  it("returns false when a property is renamed", () => {
    const stored = s({ oldName: { type: "string" } });
    const current = s({ newName: { type: "string" } });
    expect(schemaKeysMatch(stored, current)).toBe(false);
  });
});
