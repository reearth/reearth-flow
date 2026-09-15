/**
 * The two actions the nullable-enum bug was reported against, compiled from
 * the engine's real schemas.
 *
 * Each assertion here corresponds to something the patch-then-RJSF pipeline got
 * wrong in a way a user could see. `docs/schema-form-replacement.md` records the
 * measurements these come from.
 */
import { readFileSync } from "fs";

import { describe, expect, it } from "vitest";

import { compile, selectVariant } from "./compile";
import type {
  EnumField,
  ExprField,
  FieldNode,
  ObjectField,
  UnionField,
} from "./types";

const actions = JSON.parse(
  readFileSync("../engine/schema/actions.json", "utf8"),
).actions as { name: string; parameter?: unknown }[];

const formFor = (name: string): ObjectField => {
  const action = actions.find((entry) => entry.name === name);
  if (!action?.parameter) throw new Error(`${name} has no parameter schema`);
  return compile(action.parameter as never, {
    actionName: name,
  }) as ObjectField;
};

const field = (parent: ObjectField, name: string): FieldNode => {
  const found = parent.properties.find((property) => property.name === name);
  if (!found) throw new Error(`${parent.name || "root"} has no field ${name}`);
  return found;
};

describe("HTTP Caller", () => {
  const form = formFor("HTTP Caller");

  it("keeps responseEncoding an enum that may be unset", () => {
    const response = field(form, "response");
    expect(response.nullable).toBe(true);

    const encoding = field(response as ObjectField, "responseEncoding");
    expect(encoding.kind).toBe("enum");
    expect(encoding.nullable).toBe(true);
    expect((encoding as EnumField).options.map((o) => o.value)).toEqual([
      "text",
      "base64",
    ]);
    // The schema states no default; the old pipeline invented "text" here and
    // wrote it back on mount, disabling content-type sniffing.
    expect(encoding.default).toBeUndefined();
  });

  it("leaves every optional section optional", () => {
    for (const name of [
      "authentication",
      "requestBody",
      "response",
      "retry",
      "rateLimit",
      "timeouts",
      "httpOptions",
    ]) {
      expect(field(form, name).nullable, `${name} should be nullable`).toBe(
        true,
      );
      expect(form.required).not.toContain(name);
    }
    expect(form.required).toEqual(["url"]);
  });

  it("treats authentication as a union with no variant chosen when unset", () => {
    const auth = field(form, "authentication") as UnionField;
    expect(auth.kind).toBe("union");
    expect(auth.tagged).toBe(true);
    expect(auth.variants.map((variant) => variant.title)).toEqual([
      "Basic Authentication",
      "Bearer Token",
      "API Key",
    ]);
    // Nothing selected, rather than defaulting to Basic and demanding a
    // username and password the user never asked to supply.
    expect(selectVariant(auth, undefined)).toBe(-1);
    expect(selectVariant(auth, { type: "apiKey", keyName: "k" })).toBe(2);
  });

  it("hides the tag property inside each authentication variant", () => {
    const auth = field(form, "authentication") as UnionField;
    const basic = auth.variants[0].node as ObjectField;
    expect(basic.properties.map((p) => p.name)).toEqual([
      "username",
      "password",
    ]);
  });

  it("resolves the allOf sitting inside the API Key variant", () => {
    // `#Authentication.oneOf[2].location` — one of the 13 sites no old pass
    // reached, because none descended into a `oneOf` branch.
    const auth = field(form, "authentication") as UnionField;
    const apiKey = auth.variants[2].node as ObjectField;
    const location = field(apiKey, "location") as EnumField;
    expect(location.kind).toBe("enum");
    expect(location.title).toBe("Location");
    expect(location.default).toBe("header");
    expect(location.options).toEqual([
      {
        value: "header",
        label: "Header",
        description: "Include API key in HTTP header",
      },
      {
        value: "query",
        label: "Query Parameter",
        description: "Include API key in URL query string",
      },
    ]);
  });

  it("recognises url as an expression field from the schema alone", () => {
    const url = field(form, "url") as ExprField;
    expect(url.kind).toBe("expr");
    expect(url.allowedTypes).toEqual(["flowExpr", "string"]);
    expect(url.flavor).toBe("flowExpr");
  });
});

describe("Feature Writer", () => {
  const form = formFor("Feature Writer");

  it("renders format as one union, not a union plus a redundant tag dropdown", () => {
    const format = field(form, "format") as UnionField;
    expect(format.kind).toBe("union");
    expect(format.tagged).toBe(true);
    expect(format.variants.map((variant) => variant.title)).toEqual([
      "CSV",
      "TSV",
      "JSON",
    ]);
    // CSV and TSV carry nothing but their tag, so they render as a bare choice.
    expect((format.variants[0].node as ObjectField).properties).toEqual([]);
    expect((format.variants[1].node as ObjectField).properties).toEqual([]);
  });

  it("keeps the JSON converter an expression that may be unset", () => {
    const format = field(form, "format") as UnionField;
    const json = format.variants[2].node as ObjectField;
    const converter = field(json, "converter") as ExprField;

    expect(converter.kind).toBe("expr");
    expect(converter.nullable).toBe(true);
    // The engine accepts only an expression here, never a literal string.
    expect(converter.allowedTypes).toEqual(["flowExpr"]);
    expect(converter.default).toBeNull();
  });

  it("selects the stored format variant by its tag", () => {
    const format = field(form, "format") as UnionField;
    expect(selectVariant(format, { type: "csv" })).toBe(0);
    expect(selectVariant(format, { type: "json", converter: null })).toBe(2);
  });
});

describe("Python Script Processor", () => {
  it("routes script to the Python editor and nothing else", () => {
    const form = formFor("Python Script Processor");
    const script = field(form, "script") as ExprField;
    expect(script.kind).toBe("expr");
    expect(script.flavor).toBe("python");

    // The same shape under a different action stays FlowExpr.
    const url = field(formFor("HTTP Caller"), "url") as ExprField;
    expect(url.flavor).toBe("flowExpr");
  });
});
