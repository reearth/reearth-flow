import { readFileSync } from "fs";

import { describe, expect, it } from "vitest";

import {
  deleteAtPath,
  getAtPath,
  parsePathKey,
  pathKey,
  setAtPath,
} from "./path";
import { validate } from "./validate";

describe("pathKey", () => {
  it("round-trips, reading array indices back as numbers", () => {
    const path = ["operations", 0, "value"];
    expect(pathKey(path)).toBe("operations.0.value");
    expect(parsePathKey("operations.0.value")).toEqual(path);
  });

  it("treats the root as the empty key", () => {
    expect(pathKey([])).toBe("");
    expect(parsePathKey("")).toEqual([]);
  });

  it("keeps a leading-zero segment a string, since it is not an index", () => {
    expect(parsePathKey("a.007")).toEqual(["a", "007"]);
  });

  it("round-trips a map key containing a dot as one segment", () => {
    // A user-typed key like `roads.highway` used to serialise to
    // `map.roads.highway` and read back as three segments, so an edit landed on
    // a nested pair of fields and corrupted every collaborator's copy.
    const path = ["map", "roads.highway"];
    const key = pathKey(path);
    expect(parsePathKey(key)).toEqual(path);
    expect(
      getAtPath({ map: { "roads.highway": "a" } }, parsePathKey(key)),
    ).toBe("a");
    expect(
      setAtPath({ map: { "roads.highway": "a" } }, parsePathKey(key), "b"),
    ).toEqual({ map: { "roads.highway": "b" } });
  });

  it("round-trips a map key made of digits as a string, not an index", () => {
    const path = ["map", "0"];
    expect(parsePathKey(pathKey(path))).toEqual(path);
    expect(setAtPath({}, parsePathKey(pathKey(path)), "x")).toEqual({
      map: { "0": "x" },
    });
  });

  it("round-trips a map key containing the escape character itself", () => {
    for (const key of ["a~b", "a~1b", "a~0b", "~2", "a.b~c.d"]) {
      expect(parsePathKey(pathKey(["map", key]))).toEqual(["map", key]);
    }
  });

  it("leaves an array index as bare digits, keeping the wire format intact", () => {
    expect(pathKey(["rules", 0, "name"])).toBe("rules.0.name");
  });

  it("holds the assumption dot paths rest on: no '.' in any property name", () => {
    // If the engine ever emits a property name containing a dot, draft patches
    // and awareness keys become ambiguous. Fail here rather than in the field.
    const actions = JSON.parse(
      readFileSync("../engine/schema/actions.json", "utf8"),
    ).actions as { parameter?: unknown }[];

    const offenders: string[] = [];
    const walk = (node: unknown) => {
      if (!node || typeof node !== "object") return;
      const record = node as Record<string, unknown>;
      for (const group of ["properties", "definitions"]) {
        const entries = record[group];
        if (entries && typeof entries === "object") {
          for (const [key, child] of Object.entries(entries)) {
            if (group === "properties" && key.includes("."))
              offenders.push(key);
            walk(child);
          }
        }
      }
      for (const key of ["items", "additionalProperties"]) walk(record[key]);
      for (const key of ["allOf", "anyOf", "oneOf"]) {
        const branches = record[key];
        if (Array.isArray(branches)) branches.forEach(walk);
      }
    };
    actions.forEach((action) => walk(action.parameter));

    expect(offenders).toEqual([]);
  });
});

describe("validation errors reach the field they belong to", () => {
  // The field's key and the error's key are produced by different code. If only
  // one of them escapes, an error on a map entry the user named `bldg.part` is
  // filed under a key no field is listening on, and is never shown.
  it("keys an error on a dotted map key the same way the field does", () => {
    const schema = {
      type: "object",
      properties: {
        inline: {
          type: "object",
          additionalProperties: {
            type: "object",
            required: ["sourcePath"],
            properties: { sourcePath: { type: "string" } },
          },
        },
      },
    };
    const errors = validate(schema as never, {
      inline: { "bldg.part": {} },
    });
    expect(Object.keys(errors)).toContain(
      pathKey(["inline", "bldg.part", "sourcePath"]),
    );
  });

  it("still keys an array index as bare digits", () => {
    const schema = {
      type: "object",
      properties: {
        rows: {
          type: "array",
          items: {
            type: "object",
            required: ["a"],
            properties: { a: { type: "string" } },
          },
        },
      },
    };
    const errors = validate(schema as never, { rows: [{ a: "ok" }, {}] });
    expect(Object.keys(errors)).toContain(pathKey(["rows", 1, "a"]));
    expect(Object.keys(errors)).toContain("rows.1.a");
  });

  it("keys an error on a map key made of digits as a string", () => {
    const schema = {
      type: "object",
      properties: {
        inline: {
          type: "object",
          additionalProperties: { type: "string" },
        },
      },
    };
    const errors = validate(schema as never, { inline: { "0": 1 } });
    expect(Object.keys(errors)).toContain(pathKey(["inline", "0"]));
  });
});

describe("setAtPath", () => {
  it("sets a shallow value without mutating the input", () => {
    const original = { a: 1, b: 2 };
    expect(setAtPath(original, ["a"], 9)).toEqual({ a: 9, b: 2 });
    expect(original.a).toBe(1);
  });

  it("creates missing intermediate objects", () => {
    expect(setAtPath({}, ["a", "b", "c"], 1)).toEqual({ a: { b: { c: 1 } } });
  });

  it("returns the value itself for the empty path", () => {
    expect(setAtPath({ a: 1 }, [], "replaced")).toBe("replaced");
  });

  it("keeps an array an array when writing into it", () => {
    // The case a bug report pinned down: a patch into `operations.0.value`
    // used to be able to turn the array into an object keyed "0".
    const original = {
      operations: [
        { method: "create", value: "12345", attribute: "myAttribute" },
      ],
    };
    const result = setAtPath(
      original,
      parsePathKey("operations.0.value"),
      "67890",
    ) as typeof original;

    expect(Array.isArray(result.operations)).toBe(true);
    expect(result.operations).toHaveLength(1);
    expect(result.operations[0]).toEqual({
      method: "create",
      value: "67890",
      attribute: "myAttribute",
    });
  });

  it("creates an array when a numeric segment has no container yet", () => {
    const result = setAtPath({}, ["rules", 0, "name"], "first");
    expect(Array.isArray((result as { rules: unknown }).rules)).toBe(true);
    expect(result).toEqual({ rules: [{ name: "first" }] });
  });

  it("creates an object for a string segment even when it looks numeric", () => {
    const result = setAtPath({}, ["map", "0"], "value");
    expect(Array.isArray((result as { map: unknown }).map)).toBe(false);
    expect(result).toEqual({ map: { "0": "value" } });
  });

  it("handles nested arrays", () => {
    const original = { grid: [[1, 2], [3]] };
    expect(setAtPath(original, ["grid", 0, 1], 9)).toEqual({
      grid: [[1, 9], [3]],
    });
  });

  it("writes an explicit undefined rather than skipping the key", () => {
    expect(setAtPath({ a: 1 }, ["a"], undefined)).toEqual({ a: undefined });
  });
});

describe("getAtPath", () => {
  it("reads through objects and arrays", () => {
    const value = { rules: [{ name: "a" }, { name: "b" }] };
    expect(getAtPath(value, ["rules", 1, "name"])).toBe("b");
  });

  it("returns undefined for a path that does not exist", () => {
    expect(getAtPath({ a: 1 }, ["a", "b", "c"])).toBeUndefined();
    expect(getAtPath(undefined, ["a"])).toBeUndefined();
  });

  it("returns the whole value for the empty path", () => {
    const value = { a: 1 };
    expect(getAtPath(value, [])).toBe(value);
  });
});

describe("deleteAtPath", () => {
  it("removes an object key", () => {
    expect(deleteAtPath({ a: 1, b: 2 }, ["a"])).toEqual({ b: 2 });
  });

  it("removes an array element and closes the gap", () => {
    expect(deleteAtPath({ rules: ["a", "b", "c"] }, ["rules", 1])).toEqual({
      rules: ["a", "c"],
    });
  });

  it("removes a nested key without disturbing its siblings", () => {
    expect(deleteAtPath({ a: { b: 1, c: 2 } }, ["a", "b"])).toEqual({
      a: { c: 2 },
    });
  });
});
