import { RJSFSchema } from "@rjsf/utils";
import { describe, it, expect } from "vitest";

import { computeSchemaFingerprint, schemasMatch } from "./schemaFingerprint";

const s = (properties: Record<string, object>): RJSFSchema =>
  ({ properties }) as RJSFSchema;

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

  it("is invariant to property definition order", () => {
    const a = s({ alpha: { type: "string" }, beta: { type: "number" } });
    const b = s({ beta: { type: "number" }, alpha: { type: "string" } });
    expect(computeSchemaFingerprint(a)).toBe(computeSchemaFingerprint(b));
  });

  it("produces different fingerprints when keys differ", () => {
    const a = s({ a: { type: "string" } });
    const b = s({ b: { type: "string" } });
    expect(computeSchemaFingerprint(a)).not.toBe(computeSchemaFingerprint(b));
  });

  it("produces different fingerprints when a property type changes", () => {
    const a = s({ a: { type: "string" } });
    const b = s({ a: { type: "number" } });
    expect(computeSchemaFingerprint(a)).not.toBe(computeSchemaFingerprint(b));
  });

  it("produces different fingerprints when enum values change", () => {
    const a = s({ mode: { type: "string", enum: ["a", "b"] } });
    const b = s({ mode: { type: "string", enum: ["a", "b", "c"] } });
    expect(computeSchemaFingerprint(a)).not.toBe(computeSchemaFingerprint(b));
  });

  it("produces different fingerprints when nested properties change", () => {
    const a = s({
      config: {
        type: "object",
        properties: { host: { type: "string" } },
      },
    });
    const b = s({
      config: {
        type: "object",
        properties: { host: { type: "string" }, port: { type: "number" } },
      },
    });
    expect(computeSchemaFingerprint(a)).not.toBe(computeSchemaFingerprint(b));
  });

  it("produces the same fingerprint when only titles change", () => {
    const a = s({ a: { type: "string", title: "old" } });
    const b = s({ a: { type: "string", title: "new" } });
    expect(computeSchemaFingerprint(a)).toBe(computeSchemaFingerprint(b));
  });

  it("produces the same fingerprint when only descriptions change (translations)", () => {
    const a = s({ a: { type: "string", description: "The URL" } });
    const b = s({ a: { type: "string", description: "URLを入力" } });
    expect(computeSchemaFingerprint(a)).toBe(computeSchemaFingerprint(b));
  });

  it("produces the same fingerprint when nested titles/descriptions change", () => {
    const a = s({
      config: {
        type: "object",
        properties: { host: { type: "string", title: "Host", description: "x" } },
      },
    });
    const b = s({
      config: {
        type: "object",
        properties: { host: { type: "string", title: "ホスト", description: "y" } },
      },
    });
    expect(computeSchemaFingerprint(a)).toBe(computeSchemaFingerprint(b));
  });
});

describe("schemasMatch", () => {
  it("returns true when storedSchema is undefined (no baseline)", () => {
    expect(schemasMatch(undefined, s({ a: { type: "string" } }))).toBe(true);
  });

  it("returns true when schemas are identical", () => {
    const stored = s({ a: { type: "string" }, b: { type: "number" } });
    const current = s({ a: { type: "string" }, b: { type: "number" } });
    expect(schemasMatch(stored, current)).toBe(true);
  });

  it("returns true when key order differs between stored and current", () => {
    const stored = s({ a: { type: "string" }, b: { type: "number" } });
    const current = s({ b: { type: "number" }, a: { type: "string" } });
    expect(schemasMatch(stored, current)).toBe(true);
  });

  it("returns true when only translated text differs", () => {
    const stored = s({ url: { type: "string", title: "URL", description: "en" } });
    const current = s({ url: { type: "string", title: "URL先", description: "ja" } });
    expect(schemasMatch(stored, current)).toBe(true);
  });

  it("returns false when a property is added", () => {
    const stored = s({ a: { type: "string" } });
    const current = s({ a: { type: "string" }, b: { type: "number" } });
    expect(schemasMatch(stored, current)).toBe(false);
  });

  it("returns false when a property is removed", () => {
    const stored = s({ a: { type: "string" }, b: { type: "number" } });
    const current = s({ a: { type: "string" } });
    expect(schemasMatch(stored, current)).toBe(false);
  });

  it("returns false when a property is renamed", () => {
    const stored = s({ oldName: { type: "string" } });
    const current = s({ newName: { type: "string" } });
    expect(schemasMatch(stored, current)).toBe(false);
  });

  it("returns false when a property type changes", () => {
    const stored = s({ count: { type: "string" } });
    const current = s({ count: { type: "number" } });
    expect(schemasMatch(stored, current)).toBe(false);
  });

  it("returns false when required fields change", () => {
    const stored = {
      ...s({ a: { type: "string" } }),
      required: [],
    } as RJSFSchema;
    const current = {
      ...s({ a: { type: "string" } }),
      required: ["a"],
    } as RJSFSchema;
    expect(schemasMatch(stored, current)).toBe(false);
  });

  it("returns false when oneOf variants change", () => {
    const stored = s({
      source: { oneOf: [{ type: "string" }] },
    });
    const current = s({
      source: { oneOf: [{ type: "string" }, { type: "object" }] },
    });
    expect(schemasMatch(stored, current)).toBe(false);
  });
});
