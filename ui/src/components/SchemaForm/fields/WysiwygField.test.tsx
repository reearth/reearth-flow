/**
 * The rich-text field is the one place a remote edit can be dropped: applying
 * incoming content under the caret would move it mid-sentence, so an update
 * that lands while the editor has focus is deliberately skipped. It then has to
 * be applied on blur — without that the collaborator's text was never shown,
 * and the next local keystroke wrote over it.
 */
import { render, fireEvent } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { SchemaForm } from "../index";

const schema = {
  type: "object",
  properties: {
    content: { type: "string", format: "wysiwyg", title: "Content" },
  },
} as never;

const editorBody = (container: HTMLElement) =>
  container.querySelector(".ql-editor") as HTMLElement;

const renderField = (value: string) => {
  const onChange = vi.fn();
  const view = render(
    <SchemaForm
      schema={schema}
      defaultFormData={{ content: value }}
      onChange={onChange}
    />,
  );
  return { onChange, ...view };
};

describe("WysiwygField", () => {
  it("shows the value it is given", () => {
    const { container } = renderField("<p>hello</p>");
    expect(editorBody(container).textContent).toContain("hello");
  });

  it("applies an update that arrived while it had focus, once it blurs", () => {
    const { container, rerender } = renderField("<p>mine</p>");
    const body = editorBody(container);

    fireEvent.focus(body);
    // A collaborator's edit lands while the caret is in the editor.
    rerender(
      <SchemaForm
        schema={schema}
        defaultFormData={{ content: "<p>theirs</p>" }}
        onChange={() => {}}
      />,
    );
    expect(body.textContent).toContain("mine");

    fireEvent.blur(body);
    expect(editorBody(container).textContent).toContain("theirs");
  });

  it("leaves the editor alone when the value has not moved", () => {
    const { container } = renderField("<p>same</p>");
    const body = editorBody(container);

    fireEvent.focus(body);
    fireEvent.blur(body);

    expect(body.textContent).toContain("same");
  });
});
