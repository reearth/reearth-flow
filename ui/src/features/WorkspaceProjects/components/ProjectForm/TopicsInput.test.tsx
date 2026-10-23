/**
 * Topics are normalized as the server does it — trimmed, blanks and repeats
 * dropped, at most 200 of at most 64 characters — so what the user sees is
 * what gets saved.
 */
import { fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";

import { TopicsInput } from "./TopicsInput";

const mount = (initial: string[] = []) => {
  const onChange = vi.fn();
  const onDraftInvalidChange = vi.fn();
  const Harness: React.FC = () => {
    const [topics, setTopics] = useState(initial);
    return (
      <TopicsInput
        value={topics}
        onDraftInvalidChange={onDraftInvalidChange}
        onChange={(next) => {
          onChange(next);
          setTopics(next);
        }}
      />
    );
  };
  const { unmount } = render(<Harness />);
  return {
    onChange,
    onDraftInvalidChange,
    unmount,
    input: screen.getByRole("textbox"),
  };
};

const type = (input: HTMLElement, text: string, key = "Enter") => {
  fireEvent.change(input, { target: { value: text } });
  fireEvent.keyDown(input, { key });
};

describe("TopicsInput", () => {
  it("adds a trimmed topic on Enter and clears the input", () => {
    const { onChange, input } = mount();
    type(input, "  gis  ");
    expect(onChange).toHaveBeenLastCalledWith(["gis"]);
    expect(input).toHaveValue("");
  });

  it("splits a comma-separated entry and drops blanks and repeats", () => {
    const { onChange, input } = mount(["gis"]);
    type(input, "plateau, , gis,citygml, plateau");
    expect(onChange).toHaveBeenLastCalledWith(["gis", "plateau", "citygml"]);
  });

  it("adds on comma", () => {
    const { onChange, input } = mount();
    type(input, "plateau", ",");
    expect(onChange).toHaveBeenLastCalledWith(["plateau"]);
  });

  it("keeps a topic that is too long in the input", () => {
    const { onChange, input } = mount();
    type(input, "x".repeat(65));
    expect(onChange).not.toHaveBeenCalled();
    expect(input).toHaveValue("x".repeat(65));

    type(input, "x".repeat(64));
    expect(onChange).toHaveBeenLastCalledWith(["x".repeat(64)]);
  });

  it("stops at 200 topics", () => {
    const full = Array.from({ length: 200 }, (_, i) => `t${i}`);
    const { onChange, input } = mount(full);
    type(input, "one-more");
    expect(onChange).not.toHaveBeenCalled();
  });

  it("removes the last topic on Backspace in an empty input", () => {
    const { onChange, input } = mount(["gis", "plateau"]);
    fireEvent.keyDown(input, { key: "Backspace" });
    expect(onChange).toHaveBeenLastCalledWith(["gis"]);
  });

  it("removes a topic from its button", () => {
    const { onChange } = mount(["gis", "plateau"]);
    fireEvent.click(screen.getByRole("button", { name: /gis/ }));
    expect(onChange).toHaveBeenLastCalledWith(["plateau"]);
  });

  it("adds what is typed when the input loses focus", () => {
    const { onChange, input } = mount();
    fireEvent.change(input, { target: { value: "lod2" } });
    fireEvent.blur(input);
    expect(onChange).toHaveBeenLastCalledWith(["lod2"]);
  });

  it("reports text too long to add until it is shortened", () => {
    const { onDraftInvalidChange, input } = mount();
    fireEvent.change(input, { target: { value: "x".repeat(65) } });
    expect(onDraftInvalidChange).toHaveBeenLastCalledWith(true);

    fireEvent.change(input, { target: { value: "x".repeat(64) } });
    expect(onDraftInvalidChange).toHaveBeenLastCalledWith(false);
  });

  it("stops holding the form back once it is gone", () => {
    const { onDraftInvalidChange, input, unmount } = mount();
    fireEvent.change(input, { target: { value: "x".repeat(65) } });
    unmount();
    expect(onDraftInvalidChange).toHaveBeenLastCalledWith(false);
  });
});
