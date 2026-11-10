import { describe, expect, it } from "vitest";

import {
  defaultVariableName,
  findVariableNameProblems,
  hasVariableNameProblems,
  isProblemVariableName,
} from "./variableNames";

describe("defaultVariableName", () => {
  it("names a variable after its type in camelCase", () => {
    expect(defaultVariableName("choice", [])).toBe("choiceVariable");
    expect(defaultVariableName("yes_no", [])).toBe("yesNoVariable");
    expect(defaultVariableName("coordinate_system", [])).toBe(
      "coordinateSystemVariable",
    );
  });

  it("numbers it from 2 when the name is taken", () => {
    expect(defaultVariableName("text", ["textVariable"])).toBe("textVariable2");
  });

  it("takes the lowest number not in use", () => {
    expect(
      defaultVariableName("text", [
        "textVariable",
        "textVariable3",
        "textVariable4",
      ]),
    ).toBe("textVariable2");
    expect(defaultVariableName("text", ["textVariable", "textVariable2"])).toBe(
      "textVariable3",
    );
  });

  it("ignores names taken by other types", () => {
    expect(defaultVariableName("number", ["textVariable"])).toBe(
      "numberVariable",
    );
  });
});

describe("findVariableNameProblems", () => {
  it("finds nothing wrong with unique names", () => {
    const problems = findVariableNameProblems([
      { name: "cityCode" },
      { name: "cityGmlPath" },
    ]);
    expect(hasVariableNameProblems(problems)).toBe(false);
  });

  it("finds names used more than once, once each", () => {
    expect(
      findVariableNameProblems([
        { name: "a" },
        { name: "a" },
        { name: "a" },
        { name: "b" },
      ]).duplicates,
    ).toEqual(["a"]);
  });

  it("treats names that differ only in case as different", () => {
    expect(
      findVariableNameProblems([{ name: "cityCode" }, { name: "citycode" }])
        .duplicates,
    ).toEqual([]);
  });

  it("finds names with whitespace of any kind", () => {
    expect(
      findVariableNameProblems([
        { name: "New Workflow Variable" },
        { name: "tab\there" },
        { name: "fine" },
      ]).withWhitespace,
    ).toEqual(["New Workflow Variable", "tab\there"]);
  });

  it("finds empty names", () => {
    const problems = findVariableNameProblems([{ name: "" }, { name: "" }]);
    expect(problems.hasEmpty).toBe(true);
    expect(problems.duplicates).toEqual([]);
    expect(isProblemVariableName("", problems)).toBe(true);
  });

  it("tells whether a given name is a problem", () => {
    const problems = findVariableNameProblems([
      { name: "a" },
      { name: "a" },
      { name: "b" },
    ]);
    expect(isProblemVariableName("a", problems)).toBe(true);
    expect(isProblemVariableName("b", problems)).toBe(false);
  });
});
