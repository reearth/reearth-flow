import {
  getCompletionContext,
  isInsideAttributeAccessor,
  isInsideEnvAccessor,
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
    expect(isInsideAttributeAccessor('env["VA')).toBe(false);
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

describe("isInsideEnvAccessor", () => {
  test("matches an open env accessor", () => {
    expect(isInsideEnvAccessor('env["')).toBe(true);
    expect(isInsideEnvAccessor('env["BASE_')).toBe(true);
    expect(isInsideEnvAccessor("Url(env['pa")).toBe(true);
    expect(isInsideEnvAccessor('env[ "ke')).toBe(true);
  });

  test("does not match once the string is closed", () => {
    expect(isInsideEnvAccessor('env["VAR"')).toBe(false);
    expect(isInsideEnvAccessor('env["VAR"]')).toBe(false);
  });

  test("does not match a different accessor", () => {
    expect(isInsideEnvAccessor('attributes["na')).toBe(false);
    expect(isInsideEnvAccessor("env")).toBe(false);
  });

  test("does not match an identifier that merely ends in env", () => {
    expect(isInsideEnvAccessor('myenv["VA')).toBe(false);
    expect(isInsideEnvAccessor('my_env["VA')).toBe(false);
  });
});

describe("getCompletionContext env accessor", () => {
  test("reports env kind with the typed prefix", () => {
    const context = getCompletionContext('env["BASE_', 10);
    expect(context).toMatchObject({
      kind: "env",
      prefix: "BASE_",
      start: 5,
      end: 10,
      afterDot: false,
    });
  });

  test("an empty accessor yields an empty env prefix", () => {
    expect(getCompletionContext('env["', 5)).toMatchObject({
      kind: "env",
      prefix: "",
      start: 5,
    });
  });

  test("replaces the whole key when completing mid-name", () => {
    // Caret after "BA" inside an existing "BASE_DIR".
    const context = getCompletionContext('env["BASE_DIR"]', 7);
    expect(context).toMatchObject({ kind: "env", prefix: "BA", start: 5 });
    expect(context.end).toBe(13); // through "BASE_DIR", stopping at the quote
  });

  test("stays general outside an accessor", () => {
    expect(getCompletionContext("env", 3).kind).toBe("general");
    expect(getCompletionContext('myenv["VA', 9).kind).toBe("general");
  });
});
