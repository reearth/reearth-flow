import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { parseInboundMessage } from "./messages";
import { useEmbedBridge } from "./useEmbedBridge";

const DASHBOARD = "https://dashboard.example.com";

describe("parseInboundMessage", () => {
  it("accepts init with a token and theme", () => {
    expect(
      parseInboundMessage({ type: "flow:init", token: "t", theme: "light" }),
    ).toEqual({ type: "flow:init", token: "t", theme: "light" });
  });

  it("treats an empty token as no token", () => {
    expect(parseInboundMessage({ type: "flow:init", token: "" })).toEqual({
      type: "flow:init",
      token: undefined,
      theme: undefined,
    });
  });

  it("rejects malformed messages", () => {
    expect(parseInboundMessage(null)).toBeUndefined();
    expect(parseInboundMessage("flow:init")).toBeUndefined();
    expect(
      parseInboundMessage({ type: "flow:init", token: 1 }),
    ).toBeUndefined();
    expect(
      parseInboundMessage({ type: "flow:init", theme: "midnight" }),
    ).toBeUndefined();
    expect(
      parseInboundMessage({ type: "flow:open-workflow", workflowId: "" }),
    ).toBeUndefined();
    expect(parseInboundMessage({ type: "flow:unknown" })).toBeUndefined();
  });
});

describe("useEmbedBridge", () => {
  let parentFrame: HTMLIFrameElement;
  let parentWindow: Window;
  const originalParent = window.parent;

  beforeEach(() => {
    window.REEARTH_CONFIG = { reearthDashboardUrl: `${DASHBOARD}/some/path` };
    parentFrame = document.createElement("iframe");
    document.body.appendChild(parentFrame);
    parentWindow = parentFrame.contentWindow as Window;
    Object.defineProperty(window, "parent", {
      value: parentWindow,
      configurable: true,
    });
  });

  afterEach(() => {
    Object.defineProperty(window, "parent", {
      value: originalParent,
      configurable: true,
    });
    parentFrame.remove();
    window.REEARTH_CONFIG = undefined;
  });

  const dispatch = (data: unknown, origin: string, source: Window) =>
    act(() => {
      window.dispatchEvent(
        new MessageEvent("message", { data, origin, source }),
      );
    });

  it("posts only to the dashboard origin", () => {
    const postMessage = vi
      .spyOn(parentWindow, "postMessage")
      .mockImplementation(() => undefined);
    const { result } = renderHook(() => useEmbedBridge());

    expect(result.current.isConnected).toBe(true);
    result.current.post({ type: "flow:ready" });

    expect(postMessage).toHaveBeenCalledWith({ type: "flow:ready" }, DASHBOARD);
  });

  it("delivers messages from the dashboard parent", () => {
    const onMessage = vi.fn();
    renderHook(() => useEmbedBridge(onMessage));

    dispatch({ type: "flow:init", token: "abc" }, DASHBOARD, parentWindow);

    expect(onMessage).toHaveBeenCalledWith({
      type: "flow:init",
      token: "abc",
      theme: undefined,
    });
  });

  it("ignores messages from other origins or other windows", () => {
    const onMessage = vi.fn();
    renderHook(() => useEmbedBridge(onMessage));

    dispatch(
      { type: "flow:init", token: "abc" },
      "https://evil.example.com",
      parentWindow,
    );
    dispatch({ type: "flow:init", token: "abc" }, DASHBOARD, window);

    expect(onMessage).not.toHaveBeenCalled();
  });

  it("stays disconnected without a configured dashboard", () => {
    window.REEARTH_CONFIG = {};
    const postMessage = vi.spyOn(parentWindow, "postMessage");
    const { result } = renderHook(() => useEmbedBridge());

    expect(result.current.isConnected).toBe(false);
    result.current.post({ type: "flow:ready" });

    expect(postMessage).not.toHaveBeenCalled();
  });
});
