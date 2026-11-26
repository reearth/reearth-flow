import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

import LockedBadge from "./index";

const getBadge = () => {
  const badge = screen
    .getByText("Locked")
    .closest<HTMLElement>("div.rounded-xl");
  if (!badge) throw new Error("badge container not found");
  return badge;
};

const hoverPastDelay = (el: HTMLElement) => {
  fireEvent.mouseEnter(el);
  act(() => {
    vi.advanceTimersByTime(1000);
  });
};

describe("LockedBadge", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  test("offers unlock on a locked project when an unlock handler is given", () => {
    const onUnlock = vi.fn();
    render(<LockedBadge reason="locked" onUnlock={onUnlock} />);
    const badge = getBadge();

    hoverPastDelay(badge);
    fireEvent.doubleClick(badge);

    expect(onUnlock).toHaveBeenCalledOnce();
  });

  test("never offers unlock without an unlock handler", () => {
    render(<LockedBadge reason="locked" />);
    const badge = getBadge();

    hoverPastDelay(badge);

    expect(badge).toHaveClass("cursor-default");
  });

  test("shows read only to a reader", () => {
    render(<LockedBadge reason="reader" onUnlock={vi.fn()} />);

    expect(screen.getByText("Read only")).toBeInTheDocument();
  });
});
