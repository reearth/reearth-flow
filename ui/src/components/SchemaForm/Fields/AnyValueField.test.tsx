import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";

import { SchemaForm } from "../index";

import { castToType, detectValueType } from "./AnyValueField";

/**
 * Trimmed from the engine's Attribute Range Mapper schema. `defaultValue` and
 * `outputValue` are `serde_json::Value` in Rust, so schemars emits them with a
 * title and description but no `type` — the shape that used to render as
 * "Unsupported field schema … Unknown field type undefined".
 */
const schema = {
  type: "object",
  required: ["rangeTable"],
  properties: {
    defaultValue: {
      title: "Default Value",
      description: "Value written when no range matches.",
    },
    rangeTable: {
      title: "Range Lookup Table",
      type: "array",
      items: { $ref: "#/definitions/RangeEntry" },
    },
  },
  definitions: {
    RangeEntry: {
      title: "Range Entry",
      type: "object",
      required: ["from", "outputValue", "to"],
      properties: {
        from: { title: "From (Minimum)", type: "number" },
        to: { title: "To (Maximum)", type: "number" },
        outputValue: {
          title: "Output Value",
          description: "Value written to the output attribute.",
        },
      },
    },
  },
};

const renderForm = (formData: any, onChange = vi.fn()) => {
  render(
    <SchemaForm
      schema={schema}
      defaultFormData={formData}
      onChange={onChange}
    />,
  );
  return onChange;
};

describe("AnyValueField", () => {
  test("renders untyped schema nodes instead of an unsupported-field error", () => {
    renderForm({ rangeTable: [{ from: 0, to: 10, outputValue: "low" }] });

    expect(screen.queryByText(/Unsupported field/i)).toBeNull();
    expect(screen.queryByText(/Unknown field type/i)).toBeNull();
    expect(screen.getByText("Output Value")).toBeInTheDocument();
    expect(screen.getByText("Default Value")).toBeInTheDocument();
  });

  test("edits a string value in place", () => {
    const onChange = renderForm({
      rangeTable: [{ from: 0, to: 10, outputValue: "low" }],
    });

    fireEvent.change(screen.getByDisplayValue("low"), {
      target: { value: "high" },
    });

    const calls = onChange.mock.calls;
    const [formData] = calls[calls.length - 1] ?? [];
    expect(formData.rangeTable[0].outputValue).toBe("high");
  });

  test("keeps a non-string value's own type", () => {
    renderForm({ rangeTable: [{ from: 0, to: 10, outputValue: 42 }] });

    // A number arrives as a number, not stringified into a text box.
    expect(screen.getByDisplayValue("42")).toHaveAttribute("type", "number");
  });

  test("a null value opens as Null, not as an object", () => {
    // RJSF's own fallback reads `typeof null === "object"` and offers an object
    // editor. Null Attribute Mapper defaults `defaultReplacement` to null and
    // uses null in `replacement` to mean "remove the attribute", so null has to
    // survive as null.
    renderForm({ defaultValue: null, rangeTable: [] });

    expect(
      screen.getByText("No value is written for this entry."),
    ).toBeInTheDocument();
  });

  test("a value replaced from outside the field brings its own control", () => {
    // Params are collaboratively edited, so formData can change type underneath
    // an open form. The control has to follow the value, not the last local pick.
    const { rerender } = render(
      <SchemaForm
        schema={schema}
        defaultFormData={{
          rangeTable: [{ from: 0, to: 10, outputValue: "low" }],
        }}
        onChange={vi.fn()}
      />,
    );
    expect(screen.getByDisplayValue("low")).toBeInTheDocument();

    rerender(
      <SchemaForm
        schema={schema}
        defaultFormData={{ rangeTable: [{ from: 0, to: 10, outputValue: 5 }] }}
        onChange={vi.fn()}
      />,
    );
    expect(screen.getByDisplayValue("5")).toHaveAttribute("type", "number");
  });

  test("detectValueType distinguishes null from objects", () => {
    expect(detectValueType(null)).toBe("null");
    expect(detectValueType({ a: 1 })).toBe("json");
    expect(detectValueType([1, 2])).toBe("json");
    expect(detectValueType(undefined)).toBe("string");
    expect(detectValueType("x")).toBe("string");
    expect(detectValueType(0)).toBe("number");
    expect(detectValueType(false)).toBe("boolean");
  });

  test("castToType keeps what it can when the type changes", () => {
    expect(castToType("12", "number")).toBe(12);
    expect(castToType("abc", "number")).toBe(0);
    expect(castToType(7, "string")).toBe("7");
    expect(castToType({ a: 1 }, "string")).toBe('{"a":1}');
    expect(castToType("anything", "null")).toBeNull();
    expect(castToType(null, "string")).toBe("");
    expect(castToType("x", "json")).toEqual({});
    expect(castToType({ a: 1 }, "json")).toEqual({ a: 1 });
  });
});
