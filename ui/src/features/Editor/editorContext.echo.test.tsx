import { renderHook, act } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";

import type { SpotlightFollow } from "@flow/lib/yjs";
import type { AwarenessEchoMap } from "@flow/types";

import {
  EditorProvider,
  useAwarenessEcho,
  useEchoDropdown,
  useEchoFocus,
  useEchoHover,
  useEchoPresence,
  type EditorContextType,
} from "./editorContext";

const noFollow: SpotlightFollow = {
  isFollowing: false,
  openNodeId: null,
  dialog: null,
  subEditor: null,
  nodePicker: null,
  versionSnapshot: null,
  activeElements: [],
};

const wrapper =
  (value: Partial<EditorContextType>) =>
  ({ children }: { children: React.ReactNode }) => (
    <EditorProvider
      value={
        {
          isLocked: false,
          isReaderRestricted: false,
          canViewIntermediateData: true,
          ...value,
        } as EditorContextType
      }>
      {children}
    </EditorProvider>
  );

describe("awareness echo", () => {
  it("returns the users on an element", () => {
    const awarenessEchoMap: AwarenessEchoMap = {
      "version-row:2": [{ color: "#f00", userName: "Ada" }],
    };
    const { result } = renderHook(() => useAwarenessEcho("version-row:2"), {
      wrapper: wrapper({ awarenessEchoMap }),
    });
    expect(result.current).toEqual([{ color: "#f00", userName: "Ada" }]);
  });

  it("returns a stable empty list for an element nobody is on", () => {
    const { result, rerender } = renderHook(
      () => useAwarenessEcho("version-row:9"),
      { wrapper: wrapper({ awarenessEchoMap: {} }) },
    );
    const first = result.current;
    rerender();
    expect(result.current).toBe(first);
    expect(result.current).toEqual([]);
  });

  it("does not throw without a provider", () => {
    const { result } = renderHook(() => useAwarenessEcho("anything"));
    expect(result.current).toEqual([]);
  });

  it("broadcasts presence while active and clears on unmount", () => {
    const onEchoElement = vi.fn();
    const { unmount } = renderHook(
      () => useEchoPresence("dropdown:workflows", true),
      { wrapper: wrapper({ onEchoElement }) },
    );
    expect(onEchoElement).toHaveBeenCalledWith("dropdown:workflows", true);

    unmount();
    expect(onEchoElement).toHaveBeenLastCalledWith("dropdown:workflows", false);
  });

  it("does not broadcast while inactive", () => {
    const onEchoElement = vi.fn();
    renderHook(() => useEchoPresence("dropdown:workflows", false), {
      wrapper: wrapper({ onEchoElement }),
    });
    expect(onEchoElement).not.toHaveBeenCalled();
  });
});

describe("useEchoDropdown", () => {
  it("broadcasts its own open state", () => {
    const onEchoElement = vi.fn();
    const { result } = renderHook(() => useEchoDropdown("dropdown:workflows"), {
      wrapper: wrapper({ onEchoElement, spotlightFollow: noFollow }),
    });

    expect(result.current.open).toBe(false);
    act(() => result.current.onOpenChange(true));
    expect(onEchoElement).toHaveBeenCalledWith("dropdown:workflows", true);
  });

  it("opens when the spotlighted user is on it", () => {
    const spotlightFollow: SpotlightFollow = {
      ...noFollow,
      isFollowing: true,
      activeElements: ["dropdown:workflows"],
    };
    const { result } = renderHook(() => useEchoDropdown("dropdown:workflows"), {
      wrapper: wrapper({ spotlightFollow }),
    });
    expect(result.current.open).toBe(true);
  });

  it("stays closed when the spotlighted user is on a different element", () => {
    const spotlightFollow: SpotlightFollow = {
      ...noFollow,
      isFollowing: true,
      activeElements: ["dropdown:something-else"],
    };
    const { result } = renderHook(() => useEchoDropdown("dropdown:workflows"), {
      wrapper: wrapper({ spotlightFollow }),
    });
    expect(result.current.open).toBe(false);
  });

  it("ignores the element when nobody is spotlighted", () => {
    const spotlightFollow: SpotlightFollow = {
      ...noFollow,
      isFollowing: false,
      activeElements: ["dropdown:workflows"],
    };
    const { result } = renderHook(() => useEchoDropdown("dropdown:workflows"), {
      wrapper: wrapper({ spotlightFollow }),
    });
    expect(result.current.open).toBe(false);
  });
});

describe("useEchoFocus", () => {
  it("broadcasts on focus and clears on blur", () => {
    const onEchoElement = vi.fn();
    const { result } = renderHook(() => useEchoFocus("debug-var:v1"), {
      wrapper: wrapper({ onEchoElement }),
    });

    act(() => result.current.focusProps.onFocusCapture());
    expect(onEchoElement).toHaveBeenCalledWith("debug-var:v1", true);

    act(() => result.current.focusProps.onBlurCapture());
    expect(onEchoElement).toHaveBeenLastCalledWith("debug-var:v1", false);
  });
});

describe("useEchoHover", () => {
  it("broadcasts on enter and clears on leave", () => {
    const onEchoElement = vi.fn();
    const { result } = renderHook(() => useEchoHover("workflow-item:a"), {
      wrapper: wrapper({ onEchoElement }),
    });

    act(() => result.current.hoverProps.onMouseEnter());
    expect(onEchoElement).toHaveBeenCalledWith("workflow-item:a", true);

    act(() => result.current.hoverProps.onMouseLeave());
    expect(onEchoElement).toHaveBeenLastCalledWith("workflow-item:a", false);
  });
});
