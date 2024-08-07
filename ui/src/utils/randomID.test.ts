import { randomID } from "./randomID";

test("random id with a default length of 24", () => {
  expect(randomID().length).toBe(24);
});

test("random id with a specified length of 4", () => {
  expect(randomID(4).length).toBe(4);
});

test("two ids are not equal", () => {
  expect(randomID()).not.toBe(randomID());
});
