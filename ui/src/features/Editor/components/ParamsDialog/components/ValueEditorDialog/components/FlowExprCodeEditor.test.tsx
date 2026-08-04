import { fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";

import FlowExprCodeEditor from "./FlowExprCodeEditor";
import { type AutocompleteSuggestion } from "./flowExprConstants";

beforeAll(() => {
  globalThis.ResizeObserver = class {
    observe() {}
    unobserve() {}
    disconnect() {}
  } as unknown as typeof ResizeObserver;
  window.Element.prototype.scrollIntoView = () => {};
});

const attribute = (label: string): AutocompleteSuggestion => ({
  label,
  insertText: label,
  type: "attribute",
  detail: "String",
});

const ATTRIBUTES = ["alpha", "beta", "gamma"].map(attribute);

/**
 * Mirrors the real Editor: `buildReaderAttributeSuggestions` returns a new
 * array on every render, and unrelated state changes re-render the tree.
 */
const Harness = ({
  initial = "",
  attributeLabels = ["alpha", "beta", "gamma"],
  freshArrayEachRender = true,
}: {
  initial?: string;
  attributeLabels?: string[];
  freshArrayEachRender?: boolean;
}) => {
  const [value, setValue] = useState(initial);
  const [, setTick] = useState(0);
  return (
    <div>
      <button data-testid="rerender" onClick={() => setTick((t) => t + 1)}>
        unrelated
      </button>
      <FlowExprCodeEditor
        value={value}
        onChange={setValue}
        aria-label="editor"
        attributeSuggestions={
          freshArrayEachRender ? attributeLabels.map(attribute) : ATTRIBUTES
        }
      />
    </div>
  );
};

const editor = () => screen.getByLabelText("editor") as HTMLTextAreaElement;

const options = () =>
  Array.from(document.querySelectorAll("[data-testid='editor-suggestion']"));

const labels = () =>
  options().map((option) => option.querySelector(".font-medium")?.textContent);

const highlighted = () =>
  options().findIndex((option) => option.className.includes("bg-accent "));

/** Types `text` one character at a time, as the browser would. */
const type = (textarea: HTMLTextAreaElement, text: string) => {
  for (const char of text) {
    const at = textarea.selectionStart;
    const next = textarea.value.slice(0, at) + char + textarea.value.slice(at);
    fireEvent.keyDown(textarea, { key: char });
    setValueWithCaret(textarea, next, at + 1);
  }
};

const backspace = (textarea: HTMLTextAreaElement) => {
  const at = textarea.selectionStart;
  fireEvent.keyDown(textarea, { key: "Backspace" });
  setValueWithCaret(
    textarea,
    textarea.value.slice(0, at - 1) + textarea.value.slice(at),
    at - 1,
  );
};

const setValueWithCaret = (
  textarea: HTMLTextAreaElement,
  value: string,
  caret: number,
) => {
  fireEvent.change(textarea, {
    target: { value, selectionStart: caret, selectionEnd: caret },
  });
  textarea.setSelectionRange(caret, caret);
  fireEvent.select(textarea);
};

/** Moves the caret without editing, as an arrow key or a click would. */
const moveCaret = (textarea: HTMLTextAreaElement, caret: number) => {
  textarea.setSelectionRange(caret, caret);
  fireEvent.select(textarea);
};

describe("FlowExprCodeEditor autocomplete", () => {
  test("keeps the highlighted suggestion across unrelated re-renders", () => {
    render(<Harness />);
    const textarea = editor();
    type(textarea, 'attributes["');
    expect(labels()).toEqual(["alpha", "beta", "gamma"]);

    fireEvent.keyDown(textarea, { key: "ArrowDown" });
    fireEvent.keyDown(textarea, { key: "ArrowDown" });
    expect(highlighted()).toBe(2);

    fireEvent.click(screen.getByTestId("rerender"));
    expect(highlighted()).toBe(2);

    fireEvent.keyDown(textarea, { key: "Enter" });
    expect(textarea.value).toBe('attributes["gamma');
  });

  test("typing another character sends the highlight back to the top", () => {
    render(<Harness />);
    const textarea = editor();
    type(textarea, 'attributes["');

    fireEvent.keyDown(textarea, { key: "ArrowDown" });
    expect(highlighted()).toBe(1);

    type(textarea, "g"); // list narrows to a different set
    expect(labels()).toEqual(["gamma"]);
    expect(highlighted()).toBe(0);
  });

  test("a shorter attribute list arriving late keeps the highlight in range", () => {
    const Late = () => {
      const [value, setValue] = useState("");
      const [names, setNames] = useState(["alpha", "beta", "gamma"]);
      return (
        <div>
          <button data-testid="shrink" onClick={() => setNames(["alpha"])}>
            shrink
          </button>
          <FlowExprCodeEditor
            value={value}
            onChange={setValue}
            aria-label="editor"
            attributeSuggestions={names.map(attribute)}
          />
        </div>
      );
    };
    render(<Late />);
    const textarea = editor();
    type(textarea, 'attributes["');

    fireEvent.keyDown(textarea, { key: "ArrowDown" });
    fireEvent.keyDown(textarea, { key: "ArrowDown" });
    expect(highlighted()).toBe(2);

    // A schema probe lands and replaces the list without the user touching it.
    fireEvent.click(screen.getByTestId("shrink"));
    expect(labels()).toEqual(["alpha"]);
    expect(highlighted()).toBe(0);

    fireEvent.keyDown(textarea, { key: "Enter" });
    expect(textarea.value).toBe('attributes["alpha');
  });

  test("arrow keys wrap around the list", () => {
    render(<Harness />);
    const textarea = editor();
    type(textarea, 'attributes["');

    fireEvent.keyDown(textarea, { key: "ArrowUp" });
    expect(highlighted()).toBe(2);
    fireEvent.keyDown(textarea, { key: "ArrowDown" });
    expect(highlighted()).toBe(0);
  });

  test("moving the caret closes the dropdown instead of swallowing the key", () => {
    render(<Harness />);
    const textarea = editor();
    type(textarea, "st");
    expect(labels().length).toBeGreaterThan(0);

    moveCaret(textarea, 0);
    expect(labels()).toEqual([]);
  });

  test("does not claim Enter or Tab when nothing is shown", () => {
    render(<Harness />);
    const textarea = editor();
    type(textarea, "st");
    moveCaret(textarea, 0);

    const enter = fireEvent.keyDown(textarea, { key: "Enter" });
    const tab = fireEvent.keyDown(textarea, { key: "Tab" });
    // fireEvent returns false when a handler called preventDefault.
    expect(enter).toBe(true);
    expect(tab).toBe(true);
  });

  test("backspacing to an empty accessor lists every attribute again", () => {
    render(<Harness />);
    const textarea = editor();
    type(textarea, 'attributes["a');
    expect(labels()).toEqual(["alpha"]);

    backspace(textarea);
    expect(labels()).toEqual(["alpha", "beta", "gamma"]);
  });

  test("recovers after typing a prefix that matches nothing", () => {
    render(<Harness />);
    const textarea = editor();
    type(textarea, 'attributes["az');
    expect(labels()).toEqual([]);

    backspace(textarea);
    expect(labels()).toEqual(["alpha"]);
  });

  test("completing mid-word replaces the matched span only", () => {
    render(<Harness initial={'attributes["lpha"]'} />);
    const textarea = editor();
    moveCaret(textarea, 12);
    type(textarea, "a"); // -> attributes["alpha"], caret between "a" and "lpha"

    expect(labels()).toEqual(["alpha"]);
    fireEvent.keyDown(textarea, { key: "Enter" });
    expect(textarea.value).toBe('attributes["alpha"]');
  });

  test("completes attribute names containing spaces and hyphens", () => {
    render(<Harness attributeLabels={["building height", "bldg-type"]} />);
    const textarea = editor();
    type(textarea, 'attributes["building h');
    expect(labels()).toEqual(["building height"]);

    fireEvent.keyDown(textarea, { key: "Enter" });
    expect(textarea.value).toBe('attributes["building height');
  });

  test("places the caret at the {{cursor}} placeholder", () => {
    render(<Harness />);
    const textarea = editor();
    type(textarea, "attribu");

    fireEvent.keyDown(textarea, { key: "Enter" });
    expect(textarea.value).toBe('attributes[""]');
    expect(textarea.selectionStart).toBe(12);
  });

  test.each([
    ["if", "if {\n  \n} else {\n  \n}"],
    ["while", "while {\n  \n}"],
  ])("%s parks the caret on the first body line", (keyword, expected) => {
    render(<Harness />);
    const textarea = editor();
    type(textarea, keyword);

    fireEvent.keyDown(textarea, { key: "Enter" });
    expect(textarea.value).toBe(expected);
    // Inside the opening brace, past the indent, before the closing brace.
    expect(textarea.selectionStart).toBe(expected.indexOf("{") + 4);
  });

  test("chains straight into the attribute list after accepting the accessor", () => {
    render(<Harness />);
    const textarea = editor();
    type(textarea, "attribu");
    expect(labels()).toContain("attributes[]");

    fireEvent.keyDown(textarea, { key: "Enter" });
    // Caret is now between the quotes — every attribute is offered without
    // the user having to type or backspace first.
    expect(labels()).toEqual(["alpha", "beta", "gamma"]);

    fireEvent.keyDown(textarea, { key: "ArrowDown" });
    fireEvent.keyDown(textarea, { key: "Enter" });
    expect(textarea.value).toBe('attributes["beta"]');
    expect(textarea.selectionStart).toBe(16);
    // Picking the name ends the chain — the caret is still inside the accessor,
    // so the list must be closed explicitly rather than re-offered.
    expect(labels()).toEqual([]);
  });

  test("picking an attribute name closes the dropdown", () => {
    render(<Harness />);
    const textarea = editor();
    type(textarea, 'attributes["bet');
    expect(labels()).toEqual(["beta"]);

    fireEvent.keyDown(textarea, { key: "Enter" });
    expect(textarea.value).toBe('attributes["beta');
    expect(labels()).toEqual([]);
  });

  test("accepting a non-accessor suggestion leaves the dropdown closed", () => {
    render(<Harness />);
    const textarea = editor();
    type(textarea, "stri");

    fireEvent.keyDown(textarea, { key: "Enter" });
    expect(textarea.value).toBe("strip()");
    expect(labels()).toEqual([]);
  });

  test("Escape closes the dropdown and then stops intercepting", () => {
    render(<Harness />);
    const textarea = editor();
    type(textarea, 'attributes["');
    expect(labels().length).toBe(3);

    let escapedToDocument = false;
    const listener = () => {
      escapedToDocument = true;
    };
    document.addEventListener("keydown", listener);

    fireEvent.keyDown(textarea, { key: "Escape" });
    expect(labels()).toEqual([]);
    expect(escapedToDocument).toBe(false);

    // With nothing shown, Escape must reach the dialog again.
    fireEvent.keyDown(textarea, { key: "Escape" });
    expect(escapedToDocument).toBe(true);
    document.removeEventListener("keydown", listener);
  });
});
