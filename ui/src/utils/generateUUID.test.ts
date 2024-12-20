import { generateUUID } from "./generateUUID";

test("two ids are not equal", () => {
  expect(generateUUID()).not.toBe(generateUUID());
});
