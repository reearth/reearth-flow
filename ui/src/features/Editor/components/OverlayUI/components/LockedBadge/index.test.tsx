import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

import LockedBadge from "./index";

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
    const badge = screen.getByText("Locked").closest("div.rounded-xl")!;

    hoverPastDelay(badge as HTMLElement);
    fireEvent.doubleClick(badge);

    expect(onUnlock).toHaveBeenCalledOnce();
  });

  test("never offers unlock without an unlock handler", () => {
    render(<LockedBadge reason="locked" />);
    const badge = screen.getByText("Locked").closest("div.rounded-xl")!;

    hoverPastDelay(badge as HTMLElement);

    expect(badge).toHaveClass("cursor-default");
  });

  test("shows read only to a reader", () => {
    render(<LockedBadge reason="reader" onUnlock={vi.fn()} />);

    expect(screen.getByText("Read only")).toBeInTheDocument();
  });
});
