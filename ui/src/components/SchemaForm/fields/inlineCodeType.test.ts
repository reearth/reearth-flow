import { inlineCodeType } from "./inlineCodeType";

describe("inlineCodeType", () => {
  test.each([
    'variables["hello"]',
    ' variables[ "my_input_url" ] ',
    'variables.get("hello")',
    'variables[r"hello"]',
  ])("treats %s as an expression", (text) => {
    expect(inlineCodeType(text, true)).toBe("flowExpr");
  });

  test.each([
    "https://example.com/data.geojson",
    'variables["hello"] + "/a.csv"',
    'myvariables["hello"]',
    'variables["hello',
    "",
  ])("keeps %s as a string", (text) => {
    expect(inlineCodeType(text, true)).toBe("string");
  });

  test("keeps a string when the field does not allow expressions", () => {
    expect(inlineCodeType('variables["hello"]', false)).toBe("string");
  });
});
