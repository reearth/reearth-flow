import { renderHook } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";

import useFollowSync from "./useFollowSync";

/**
 * Drives the hook the way a real consumer does: `local` is whatever `onFollow`
 * last applied, so the hook sees its own effects on the next render.
 */
const renderFollow = (initial: {
  active: boolean;
  follow: string | null;
  local?: string | null;
}) => {
  const onFollow = vi.fn();
  const view = renderHook(
    (props: { active: boolean; follow: string | null; local: string | null }) =>
      useFollowSync<string>({ ...props, onFollow }),
    {
      initialProps: {
        active: initial.active,
        follow: initial.follow,
        local: initial.local ?? null,
      },
    },
  );
  return { onFollow, ...view };
};

describe("useFollowSync", () => {
  it("applies the followed value when it differs from local", () => {
    const { onFollow } = renderFollow({ active: true, follow: "node-a" });
    expect(onFollow).toHaveBeenCalledWith("node-a");
  });

  it("does nothing when local already matches", () => {
    const { onFollow } = renderFollow({
      active: true,
      follow: "node-a",
      local: "node-a",
    });
    expect(onFollow).not.toHaveBeenCalled();
  });

  it("ignores the followed value while inactive", () => {
    const { onFollow } = renderFollow({ active: false, follow: "node-a" });
    expect(onFollow).not.toHaveBeenCalled();
  });

  it("closes what the follow opened when the followed user closes it", () => {
    const { onFollow, rerender } = renderFollow({
      active: true,
      follow: "node-a",
    });
    expect(onFollow).toHaveBeenCalledWith("node-a");

    rerender({ active: true, follow: null, local: "node-a" });
    expect(onFollow).toHaveBeenLastCalledWith(null);
  });

  it("leaves alone what the follower already had open", () => {
    const { onFollow, rerender } = renderFollow({
      active: true,
      follow: null,
      local: "node-mine",
    });
    rerender({ active: true, follow: null, local: "node-mine" });
    expect(onFollow).not.toHaveBeenCalled();
  });

  it("keeps followed content open when the spotlight is dropped", () => {
    const { onFollow, rerender } = renderFollow({
      active: true,
      follow: "node-a",
    });
    onFollow.mockClear();

    // Spotlight dropped: `follow` clears and `active` goes false in one update.
    rerender({ active: false, follow: null, local: "node-a" });
    expect(onFollow).not.toHaveBeenCalled();
  });

  it("does not re-close after the spotlight is dropped and re-taken", () => {
    const { onFollow, rerender } = renderFollow({
      active: true,
      follow: "node-a",
    });
    rerender({ active: false, follow: null, local: "node-a" });
    onFollow.mockClear();

    // A new spotlight on someone with nothing open must not close the dialog
    // the follower is now driving themselves.
    rerender({ active: true, follow: null, local: "node-a" });
    expect(onFollow).not.toHaveBeenCalled();
  });

  it("switches to a new followed value", () => {
    const { onFollow, rerender } = renderFollow({
      active: true,
      follow: "node-a",
    });
    rerender({ active: true, follow: "node-b", local: "node-a" });
    expect(onFollow).toHaveBeenLastCalledWith("node-b");
  });

  it("honours a custom equality check", () => {
    const onFollow = vi.fn();
    renderHook(() =>
      useFollowSync<{ id: string }>({
        active: true,
        follow: { id: "x" },
        local: { id: "x" },
        onFollow,
        isSame: (a, b) => a.id === b.id,
      }),
    );
    expect(onFollow).not.toHaveBeenCalled();
  });
});
