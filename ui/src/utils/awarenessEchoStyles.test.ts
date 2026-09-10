import { describe, it, expect } from "vitest";

import {
  awarenessEchoStripe,
  awarenessEchoStyles,
} from "./awarenessEchoStyles";

const ada = { color: "#ff0000", userName: "Ada" };
const linus = { color: "#00ff00", userName: "Linus" };

describe("awarenessEchoStyles", () => {
  it("rings in the first user's colour", () => {
    expect(awarenessEchoStyles([ada])).toEqual({
      boxShadow: "inset 0 0 0 2px #ff0000",
      borderRadius: "4px",
    });
  });

  it("is undefined when nobody is present", () => {
    expect(awarenessEchoStyles([])).toBeUndefined();
    expect(awarenessEchoStyles(undefined)).toBeUndefined();
    expect(awarenessEchoStyles(null)).toBeUndefined();
  });
});

describe("awarenessEchoStripe", () => {
  it("marks a list row with a left stripe, not a ring", () => {
    // A ring around a row reads as "selected in their colour" and fights the
    // row's own selected background, which follow puts on the same row.
    const style = awarenessEchoStripe([ada]);
    expect(style).toEqual({ boxShadow: "inset 3px 0 0 0 #ff0000" });
    expect(style?.boxShadow).not.toContain("0 0 0 2px");
  });

  it("uses the first present user when several share a row", () => {
    expect(awarenessEchoStripe([linus, ada])?.boxShadow).toContain("#00ff00");
  });

  it("is undefined when nobody is present", () => {
    expect(awarenessEchoStripe([])).toBeUndefined();
    expect(awarenessEchoStripe(undefined)).toBeUndefined();
  });
});
