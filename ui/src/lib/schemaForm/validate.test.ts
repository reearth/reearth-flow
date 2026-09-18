/**
 * Error pruning, against the engine's real schemas.
 *
 * A choice that fails is reported by AJV once for the choice and again for
 * every branch it tried. What reaches the user should be the complaint from
 * the field they are actually filling in, not the shallow one above it — and
 * the root is a field like any other, since four actions have a union there.
 */
import { readFileSync } from "fs";

import { describe, expect, it } from "vitest";

import { validate } from "./validate";

const actions = JSON.parse(
  readFileSync("../engine/schema/actions.json", "utf8"),
).actions as { name: string; parameter?: any }[];

const schemaFor = (name: string) => {
  const action = actions.find((entry) => entry.name === name);
  if (!action?.parameter) throw new Error(`${name} has no parameter schema`);
  return action.parameter;
};

describe("root-level pruning", () => {
  const rootUnions = actions.filter(
    (action) =>
      action.parameter &&
      !action.parameter.properties &&
      (action.parameter.oneOf || action.parameter.anyOf),
  );

  it("still finds the four union-rooted actions", () => {
    expect(rootUnions.map((action) => action.name).sort()).toEqual([
      "Feature Reader",
      "JSON Fragmenter",
      "PLATEAU4.SolarPositionCalculator",
      "XML Fragmenter",
    ]);
  });

  it.each(rootUnions.map((action) => action.name))(
    "%s drops the choice message once a field beneath it has one",
    (name) => {
      const errors = validate(schemaFor(name), {});

      // The root is still invalid — the key's presence is what says so.
      expect(errors).toHaveProperty("");
      expect(Object.keys(errors).length).toBeGreaterThan(1);

      // ...but says nothing, because the fields underneath say it better.
      // The root key is `""`, whose children are not prefixed by a dot, so
      // testing `startsWith(".")` matched nothing and left this message here.
      expect(errors[""]).not.toContain("Choose one of the available options");
    },
  );

  it("keeps the choice message when there is nothing deeper to go on", () => {
    // A scalar matches no branch and produces no child errors, so the root's
    // own report is all the user has.
    const errors = validate(schemaFor("Feature Reader"), "banana");

    expect(Object.keys(errors)).toEqual([""]);
    expect(errors[""]).toContain("Choose one of the available options");
  });

  it("marks a required field the user has not filled in, without a sentence", () => {
    const errors = validate(schemaFor("Feature Reader"), {});

    // A missing required property is keyed onto the property itself, with an
    // empty list: the asterisk already says it.
    expect(errors.dataset).toEqual([]);
    expect(errors.format).toEqual([]);
  });
});
