import {
  getCompletionContext,
  isInsideAttributeAccessor,
  isInsideVariableAccessor,
} from "./flowExprAttributeContext";

describe("isInsideAttributeAccessor", () => {
  test("matches inside an empty attributes accessor", () => {
    expect(isInsideAttributeAccessor('attributes["')).toBe(true);
  });

  test("matches while typing an attribute name", () => {
    expect(isInsideAttributeAccessor('attributes["na')).toBe(true);
  });

  test("matches single-quoted accessor and surrounding expression", () => {
    expect(isInsideAttributeAccessor("Url(attributes['pa")).toBe(true);
  });

  test("tolerates whitespace inside the brackets", () => {
    expect(isInsideAttributeAccessor('attributes[ "ke')).toBe(true);
  });

  test("does not match once the string is closed", () => {
    expect(isInsideAttributeAccessor('attributes["name"')).toBe(false);
    expect(isInsideAttributeAccessor('attributes["name"]')).toBe(false);
  });

  test("does not match outside an attributes accessor", () => {
    expect(isInsideAttributeAccessor('variables["VA')).toBe(false);
    expect(isInsideAttributeAccessor("attributes")).toBe(false);
    expect(isInsideAttributeAccessor("")).toBe(false);
  });
});

describe("getCompletionContext", () => {
  test("filters on the typed prefix but replaces the whole name", () => {
    // Caret sits after "al" inside an existing "alpha".
    const context = getCompletionContext('attributes["alpha"]', 14);
    expect(context).toMatchObject({ kind: "attribute", prefix: "al" });
    expect(context.start).toBe(12);
    expect(context.end).toBe(17); // through "alpha", stopping at the quote
  });

  test("replaces to the end of the identifier segment, not past the dot", () => {
    const context = getCompletionContext("value.split", 9); // caret in "spl|it"
    expect(context).toMatchObject({ prefix: "spl", start: 6, end: 11 });
  });

  test("attribute names are delimited by the quote, not by word characters", () => {
    const context = getCompletionContext('attributes["building h', 22);
    expect(context.kind).toBe("attribute");
    expect(context.prefix).toBe("building h");
    expect(context.start).toBe(12);
  });

  test("an empty accessor yields an empty attribute prefix", () => {
    const context = getCompletionContext('attributes["', 12);
    expect(context).toMatchObject({ kind: "attribute", prefix: "", start: 12 });
  });

  test("bare identifiers use word boundaries", () => {
    const context = getCompletionContext('Url("x") + stri', 15);
    expect(context).toMatchObject({
      kind: "general",
      prefix: "stri",
      start: 11,
      end: 15,
      afterDot: false,
    });
  });

  test("member access matches only the segment after the dot", () => {
    const context = getCompletionContext("value.spl", 9);
    expect(context).toMatchObject({
      kind: "general",
      prefix: "spl",
      start: 6,
      afterDot: true,
    });
  });

  test("clamps a caret past the end of the text", () => {
    expect(getCompletionContext("ab", 99)).toMatchObject({
      prefix: "ab",
      end: 2,
    });
  });
});

describe("isInsideVariableAccessor", () => {
  test("matches an open variables accessor", () => {
    expect(isInsideVariableAccessor('variables["')).toBe(true);
    expect(isInsideVariableAccessor('variables["BASE_')).toBe(true);
    expect(isInsideVariableAccessor("Url(variables['pa")).toBe(true);
    expect(isInsideVariableAccessor('variables[ "ke')).toBe(true);
  });

  test("does not match once the string is closed", () => {
    expect(isInsideVariableAccessor('variables["VAR"')).toBe(false);
    expect(isInsideVariableAccessor('variables["VAR"]')).toBe(false);
  });

  test("does not match a different accessor", () => {
    expect(isInsideVariableAccessor('attributes["na')).toBe(false);
    expect(isInsideVariableAccessor("variables")).toBe(false);
  });

  test("does not match an identifier that merely ends in variables", () => {
    expect(isInsideVariableAccessor('myvariables["VA')).toBe(false);
    expect(isInsideVariableAccessor('my_variables["VA')).toBe(false);
  });
});

describe("getCompletionContext variables accessor", () => {
  test("reports variables kind with the typed prefix", () => {
    const context = getCompletionContext('variables["BASE_', 16);
    expect(context).toMatchObject({
      kind: "variables",
      prefix: "BASE_",
      start: 11,
      end: 16,
      afterDot: false,
    });
  });

  test("an empty accessor yields an empty variables prefix", () => {
    expect(getCompletionContext('variables["', 11)).toMatchObject({
      kind: "variables",
      prefix: "",
      start: 11,
    });
  });

  test("replaces the whole key when completing mid-name", () => {
    // Caret after "BA" inside an existing "BASE_DIR".
    const context = getCompletionContext('variables["BASE_DIR"]', 13);
    expect(context).toMatchObject({
      kind: "variables",
      prefix: "BA",
      start: 11,
    });
    expect(context.end).toBe(19); // through "BASE_DIR", stopping at the quote
  });

  test("stays general outside an accessor", () => {
    expect(getCompletionContext("variables", 9).kind).toBe("general");
    expect(getCompletionContext('myvariables["VA', 15).kind).toBe("general");
  });
});
