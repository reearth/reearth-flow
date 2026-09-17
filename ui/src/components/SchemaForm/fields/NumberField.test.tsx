/**
 * A number on its way to being typed passes through states that are not
 * numbers — `-`, `1.`, `1e`.
 *
 * A `type="number"` input reports those as an empty `value` with
 * `validity.badInput` set, holding the characters in a buffer nothing can read.
 * jsdom implements the sanitising but not `badInput`, so the browser's report
 * is stubbed where a test needs it; everything else runs for real.
 */
import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";

import { describe, expect, it, vi } from "vitest";

import { SchemaForm } from "../index";

const schema = {
  type: "object",
  properties: {
    amount: { type: "number", title: "Amount" },
    count: { type: "integer", title: "Count" },
  },
} as never;

const mount = (initial: Record<string, unknown> = {}) => {
  const onChange = vi.fn();
  const Harness: React.FC = () => {
    const [data, setData] = useState<unknown>(initial);
    return (
      <SchemaForm
        schema={schema}
        defaultFormData={data}
        onChange={(next, key) => {
          onChange(next, key);
          setData(next);
        }}
      />
    );
  };
  return { onChange, ...render(<Harness />) };
};

const lastValue = (onChange: ReturnType<typeof vi.fn>) => {
  const calls = onChange.mock.calls;
  return calls.length
    ? (calls[calls.length - 1][0] as Record<string, unknown>)
    : undefined;
};

/** What a real number input reports for an entry it cannot parse. */
const typePartial = (input: HTMLInputElement) => {
  Object.defineProperty(input, "validity", {
    configurable: true,
    value: { badInput: true },
  });
  fireEvent.change(input, { target: { value: "" } });
};

/** …and what it reports once the entry parses again. */
const typeComplete = (input: HTMLInputElement, text: string) => {
  Object.defineProperty(input, "validity", {
    configurable: true,
    value: { badInput: false },
  });
  fireEvent.change(input, { target: { value: text } });
};

describe("NumberField", () => {
  it("keeps the native control", () => {
    mount();
    const input = screen.getByLabelText("Amount") as HTMLInputElement;
    expect(input.type).toBe("number");
    expect(input.step).toBe("any");
    expect((screen.getByLabelText("Count") as HTMLInputElement).step).toBe("1");
  });

  it("renders nothing over a half-typed number, so the input keeps it", () => {
    const { container } = mount({ amount: 5 });
    const input = screen.getByLabelText("Amount") as HTMLInputElement;

    typePartial(input);

    // Rendering the stored 5 here would reset the control and take the
    // half-typed minus with it.
    expect(
      (container.querySelector(`#${input.id}`) as HTMLInputElement).value,
    ).toBe("");
  });

  it("leaves the stored value alone while the entry is unparseable", () => {
    const { onChange } = mount({ amount: 5 });
    const input = screen.getByLabelText("Amount") as HTMLInputElement;

    typePartial(input);

    expect(onChange).not.toHaveBeenCalled();
  });

  it("stores the number once the entry becomes one", () => {
    const { onChange } = mount({ amount: 5 });
    const input = screen.getByLabelText("Amount") as HTMLInputElement;

    typePartial(input);
    typeComplete(input, "-3");

    expect(lastValue(onChange)).toEqual({ amount: -3 });
  });

  it("still clears the field when the user really empties it", () => {
    const { onChange } = mount({ amount: 5 });
    const input = screen.getByLabelText("Amount") as HTMLInputElement;

    typeComplete(input, "");

    expect(lastValue(onChange)).toEqual({});
  });

  it("drops a half-typed entry on blur and shows what was stored", () => {
    mount({ amount: 7 });
    const input = screen.getByLabelText("Amount") as HTMLInputElement;

    typePartial(input);
    expect(input.value).toBe("");

    fireEvent.blur(input);
    expect(input.value).toBe("7");
  });

  it("types a negative number end to end", async () => {
    const { onChange } = mount();
    await userEvent.type(screen.getByLabelText("Amount"), "-12.5");
    expect(lastValue(onChange)).toEqual({ amount: -12.5 });
  });

  it("parses an integer field as an integer", async () => {
    const { onChange } = mount();
    await userEvent.type(screen.getByLabelText("Count"), "42");
    expect(lastValue(onChange)).toEqual({ count: 42 });
  });

  it("reports a value outside the schema's range", () => {
    const onValidationChange = vi.fn();
    render(
      <SchemaForm
        schema={
          {
            type: "object",
            properties: { n: { type: "integer", title: "N", minimum: 0 } },
          } as never
        }
        defaultFormData={{ n: -5 }}
        onChange={() => {}}
        onValidationChange={onValidationChange}
      />,
    );
    expect(onValidationChange).toHaveBeenLastCalledWith(false);
  });
});
