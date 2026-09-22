/**
 * The Python editor edits a `format: "code"` field, which the engine's schema
 * says is `{ type, value }` — never a bare string.
 */
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import PythonEditorDialog from "./index";

vi.mock("@monaco-editor/react", () => ({
  Editor: ({
    value,
    onChange,
  }: {
    value: string;
    onChange: (next: string) => void;
  }) => (
    <textarea
      aria-label="editor"
      value={value}
      onChange={(event) => onChange(event.target.value)}
    />
  ),
}));

const context = (value: unknown) => ({
  key: "script",
  name: "script",
  path: ["script"],
  value,
  schema: { title: "Inline Script" },
  fieldName: "script",
});

const open = (value: unknown) => {
  const onValueSubmit = vi.fn();
  render(
    <PythonEditorDialog
      open
      fieldContext={context(value) as never}
      onClose={() => {}}
      onValueSubmit={onValueSubmit}
    />,
  );
  return onValueSubmit;
};

describe("PythonEditorDialog", () => {
  it("shows the script, not the object wrapping it", () => {
    open({ type: "string", value: "print('hi')" });
    expect(screen.getByLabelText("editor")).toHaveValue("print('hi')");
  });

  it("submits the object shape the schema requires", async () => {
    const onValueSubmit = open({ type: "string", value: "print('hi')" });

    await userEvent.clear(screen.getByLabelText("editor"));
    await userEvent.type(screen.getByLabelText("editor"), "x = 1");
    await userEvent.click(
      screen.getByRole("button", { name: /apply|submit/i }),
    );

    expect(onValueSubmit).toHaveBeenCalledWith({
      type: "string",
      value: "x = 1",
    });
  });

  it("keeps a script that was written as an expression an expression", async () => {
    const onValueSubmit = open({ type: "flowExpr", value: "a" });

    await userEvent.click(
      screen.getByRole("button", { name: /apply|submit/i }),
    );

    expect(onValueSubmit).toHaveBeenCalledWith({
      type: "flowExpr",
      value: "a",
    });
  });

  it("accepts a bare string left by older data", () => {
    open("legacy script");
    expect(screen.getByLabelText("editor")).toHaveValue("legacy script");
  });

  it("starts empty when the field is unset", () => {
    open(undefined);
    expect(screen.getByLabelText("editor")).toHaveValue("");
  });
});
