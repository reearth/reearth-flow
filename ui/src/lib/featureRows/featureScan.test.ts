import { describe, expect, test } from "vitest";

import { scanFeature } from "./featureScan";

const encoder = new TextEncoder();
const decoder = new TextDecoder();

const scan = (text: string) => {
  const bytes = encoder.encode(text);
  const spans = scanFeature(bytes, 0, bytes.length);
  if (!spans) return null;
  const read = (span: [number, number] | null) =>
    span ? JSON.parse(decoder.decode(bytes.subarray(...span))) : undefined;
  return { id: read(spans.id), attributes: read(spans.attributes) };
};

describe("scanFeature", () => {
  test("finds the top-level id and attributes, wherever they are", () => {
    expect(
      scan(
        '{ "geometry": {"attributes": {"x": 1}, "id": 2}, "attributes": {"a": [1, {"b": "}"}]}, "id": "f" }',
      ),
    ).toEqual({ id: "f", attributes: { a: [1, { b: "}" }] } });
  });

  test("recognizes escaped spellings of the keys", () => {
    expect(scan('{"\\u0069d":1,"\\u0061ttributes":{"k":"v"}}')).toEqual({
      id: 1,
      attributes: { k: "v" },
    });
  });

  test("accepts everything JSON.parse accepts", () => {
    const valid = [
      '{"id":-0.5e+10,"attributes":{"s":"\\"\\\\\\/\\b\\f\\n\\r\\t\\u00e9"},"g":[true,false,null,[],{}]}',
      '{"attributes":null}',
      "{}",
      '  {"id": 0}  ',
    ];
    for (const text of valid) {
      expect(() => JSON.parse(text)).not.toThrow();
      expect(scan(text)).not.toBeNull();
    }
  });

  test("rejects what JSON.parse rejects, up to the fields it reads", () => {
    const invalid = [
      '{"id":,"attributes":{"name":"A"}}',
      '{"attributes":{"name":"A",}, "id": 1}',
      '{"id":01}',
      '{"id":1.}',
      '{"id":"\\x"}',
      '{"id":"a\tb"}',
      '{"id":tru}',
      '{"id":1,}',
      '{"id":1} x',
    ];
    for (const text of invalid) {
      expect(() => JSON.parse(text)).toThrow();
      expect(scan(text)).toBeNull();
    }
    expect(scan("[1]")).toBeNull();
  });

  test("stops once both keys are found, before the geometry", () => {
    expect(scan('{"id":1,"attributes":{},"geometry":[1,')).toEqual({
      id: 1,
      attributes: {},
    });
  });
});
