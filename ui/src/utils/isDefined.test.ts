import { isDefined } from "./isDefined";

test("check if not defined", () => {
  const a: number | undefined = undefined;
  expect(isDefined(a)).toBe(false);
});

test("check if defined", () => {
  const a: number | undefined = 1;
  expect(isDefined(a)).toBe(true);
});
