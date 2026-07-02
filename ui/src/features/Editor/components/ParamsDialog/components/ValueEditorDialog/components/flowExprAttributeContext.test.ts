import { isInsideAttributeAccessor } from "./flowExprAttributeContext";

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
