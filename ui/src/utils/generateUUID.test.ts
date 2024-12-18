import { generateUUID } from "./generateUUID";

test("random id with a default length of 24", () => {
  expect(generateUUID().length).toBe(24);
});

test("two ids are not equal", () => {
  expect(generateUUID()).not.toBe(generateUUID());
});
