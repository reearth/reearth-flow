/**
 * A colour input always holds a colour, so "no colour" has to be drawn and
 * offered by the field itself. Without it a note's background, once picked,
 * could never go back to the canvas's own note background.
 */
import { fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";

import { SchemaForm } from "../index";

const schema = {
  type: "object",
  properties: {
    background: { type: "string", format: "color", title: "Background" },
    title: {
      type: "string",
      format: "color",
      title: "Title",
      default: "#fafafa",
    },
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

const clearButtons = () => screen.queryAllByRole("button", { name: "Clear" });

describe("ColorField", () => {
  it("shows an optional colour with no default as unset", () => {
    mount();
    expect(document.querySelectorAll("[data-unset]")).toHaveLength(1);
    expect(clearButtons()).toHaveLength(1);
    expect(clearButtons()[0]).toBeDisabled();
  });

  it("clears a chosen colour by removing its key", () => {
    const { onChange } = mount({ background: "#ff0000" });
    expect(document.querySelector("[data-unset]")).toBeNull();

    fireEvent.click(clearButtons()[0]);

    const last = onChange.mock.calls.at(-1)?.[0] as Record<string, unknown>;
    expect(last).not.toHaveProperty("background");
    expect(document.querySelector("[data-unset]")).not.toBeNull();
  });

  it("offers no clear on a colour the schema gives a default", () => {
    // Only the default-less field has a Clear; the other would be re-seeded.
    mount({ background: "#ff0000", title: "#000000" });
    expect(clearButtons()).toHaveLength(1);
  });
});
