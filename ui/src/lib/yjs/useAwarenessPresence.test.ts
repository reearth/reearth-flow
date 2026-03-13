import { renderHook, act } from "@testing-library/react";
import { MouseEvent } from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import type { Awareness } from "y-protocols/awareness";

import useAwarenessPresence from "./useAwarenessPresence";

// Mock dependencies
vi.mock("@xyflow/react", () => ({
  useReactFlow: () => ({
    screenToFlowPosition: ({ x, y }: { x: number; y: number }) => ({ x, y }),
  }),
  useViewport: () => ({ x: 0, y: 0, zoom: 1 }),
}));

vi.mock("y-presence", () => ({
  useSelf: (awareness: any) => awareness._mockSelf,
  useUsers: (awareness: any) => awareness._mockUsers,
}));

const createMockAwareness = () => {
  const state: Record<string, any> = {};
  return {
    clientID: 1,
    setLocalStateField: vi.fn((key, value) => {
      state[key] = value;
    }),
    getStates: vi.fn(
      () =>
        new Map([
          [1, {}],
          [2, {}],
        ]),
    ),
    _mockSelf: {
      clientID: 1,
      userName: "TestUser",
      color: "#123456",
      cursor: { x: 10, y: 20 },
      viewport: { x: 0, y: 0, zoom: 1 },
      currentWorkflowId: "wf1",
      selectionRect: { startX: 0, startY: 0, currentX: 10, currentY: 20 },
    },
    _mockUsers: new Map([
      [
        2,
        {
          clientId: 2,
          userName: "OtherUser",
          color: "#654321",
          cursor: { x: 30, y: 40 },
          viewport: { x: 0, y: 0, zoom: 1 },
          currentWorkflowId: "wf2",
          selectionRect: { startX: 0, startY: 0, currentX: 30, currentY: 40 },
        },
      ],
    ]),
  } as unknown as Awareness;
};

describe("useAwarenessPresence", () => {
  let awareness: Awareness;

  beforeEach(() => {
    awareness = createMockAwareness();
  });

  it("returns self and users correctly", () => {
    const { result } = renderHook(() =>
      useAwarenessPresence({ yAwareness: awareness }),
    );
    expect(result.current.self.userName).toBe("TestUser");
    expect(result.current.self.color).toBe("#123456");
    expect(result.current.users["2"].userName).toBe("OtherUser");
  });

  it("handlePointerDown sets cursor and viewport", () => {
    const { result } = renderHook(() =>
      useAwarenessPresence({ yAwareness: awareness }),
    );
    const event = {
      clientX: 50,
      clientY: 60,
      shiftKey: false,
      button: 0,
      preventDefault: vi.fn(),
      stopPropagation: vi.fn(),
    } as unknown as MouseEvent;
    act(() => {
      result.current.handlePointerDown(event);
    });
    expect(awareness.setLocalStateField).toHaveBeenCalledWith("cursor", {
      x: 50,
      y: 60,
    });
    expect(awareness.setLocalStateField).toHaveBeenCalledWith("viewport", {
      x: 0,
      y: 0,
      zoom: 1,
    });
  });

  it("handlePointerDown with shiftKey sets selectionRect", () => {
    const { result } = renderHook(() =>
      useAwarenessPresence({ yAwareness: awareness }),
    );
    const event = {
      clientX: 10,
      clientY: 20,
      shiftKey: true,
      button: 0,
      preventDefault: vi.fn(),
      stopPropagation: vi.fn(),
    } as unknown as MouseEvent;
    act(() => {
      result.current.handlePointerDown(event);
    });
    expect(result.current.isSelectingRef.current).toBe(true);
    expect(result.current.selectionStartRef.current).toEqual({ x: 10, y: 20 });
    expect(awareness.setLocalStateField).toHaveBeenCalledWith("selectionRect", {
      startX: 10,
      startY: 20,
      currentX: 10,
      currentY: 20,
    });
  });

  it("setDraggingEdge sets draggingEdge field", () => {
    const { result } = renderHook(() =>
      useAwarenessPresence({ yAwareness: awareness }),
    );
    act(() => {
      result.current.setDraggingEdge("node1", "handle1", "source");
    });
    expect(awareness.setLocalStateField).toHaveBeenCalledWith("draggingEdge", {
      nodeId: "node1",
      handleId: "handle1",
      handleType: "source",
    });
  });

  it("clearDraggingEdge clears draggingEdge field", () => {
    const { result } = renderHook(() =>
      useAwarenessPresence({ yAwareness: awareness }),
    );
    act(() => {
      result.current.clearDraggingEdge();
    });
    expect(awareness.setLocalStateField).toHaveBeenCalledWith(
      "draggingEdge",
      null,
    );
  });

  it("setSelectedNodes sets selectedNodeIds field", () => {
    const { result } = renderHook(() =>
      useAwarenessPresence({ yAwareness: awareness }),
    );
    act(() => {
      result.current.setSelectedNodes(["node1", "node2"]);
    });
    expect(awareness.setLocalStateField).toHaveBeenCalledWith(
      "selectedNodeIds",
      ["node1", "node2"],
    );
    act(() => {
      result.current.setSelectedNodes([]);
    });
    expect(awareness.setLocalStateField).toHaveBeenCalledWith(
      "selectedNodeIds",
      null,
    );
  });
});
