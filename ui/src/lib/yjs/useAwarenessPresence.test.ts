import { renderHook, act } from "@testing-library/react";
import { OnConnectStartParams } from "@xyflow/react";
import { MouseEvent } from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import type { Awareness } from "y-protocols/awareness";

import { Node } from "@flow/types";

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
    getLocalState: vi.fn(() => state),
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
      useAwarenessPresence({
        yAwareness: awareness,
        selectedNodeIds: [],
        openNode: undefined,
      }),
    );
    expect(result.current.self.userName).toBe("TestUser");
    expect(result.current.self.color).toBe("#123456");
    expect(result.current.users["2"].userName).toBe("OtherUser");
  });

  it("handlePointerDown sets cursor and viewport", () => {
    const { result } = renderHook(() =>
      useAwarenessPresence({
        yAwareness: awareness,
        selectedNodeIds: [],
        openNode: undefined,
      }),
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
      useAwarenessPresence({
        yAwareness: awareness,
        selectedNodeIds: [],
        openNode: undefined,
      }),
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
    expect(awareness.setLocalStateField).toHaveBeenCalledWith("selectionRect", {
      startX: 10,
      startY: 20,
      currentX: 10,
      currentY: 20,
    });
  });

  it("handleConnectStart sets draggingEdge field", () => {
    const { result } = renderHook(() =>
      useAwarenessPresence({
        yAwareness: awareness,
        selectedNodeIds: [],
        openNode: undefined,
      }),
    );
    const event = {
      clientX: 10,
      clientY: 20,
      shiftKey: true,
      button: 0,
      preventDefault: vi.fn(),
      stopPropagation: vi.fn(),
    } as unknown as globalThis.MouseEvent | TouchEvent;

    const startParams = {
      nodeId: "node1",
      handleId: "handle1",
      handleType: "source",
    } as OnConnectStartParams;

    act(() => {
      result.current.handleConnectStart(event, startParams);
    });
    expect(awareness.setLocalStateField).toHaveBeenCalledWith("draggingEdge", {
      nodeId: "node1",
      handleId: "handle1",
      handleType: "source",
    });
  });

  it("handleConnectEnd clears draggingEdge field", () => {
    const { result } = renderHook(() =>
      useAwarenessPresence({
        yAwareness: awareness,
        selectedNodeIds: [],
        openNode: undefined,
      }),
    );
    act(() => {
      result.current.handleConnectEnd();
    });
    expect(awareness.setLocalStateField).toHaveBeenCalledWith(
      "draggingEdge",
      null,
    );
  });

  it("syncs selectedNodeIds prop to awareness field", () => {
    const { rerender } = renderHook(
      ({ selectedNodeIds }: { selectedNodeIds: string[] }) =>
        useAwarenessPresence({
          yAwareness: awareness,
          selectedNodeIds,
          openNode: undefined,
        }),
      { initialProps: { selectedNodeIds: ["node1", "node2"] } },
    );
    expect(awareness.setLocalStateField).toHaveBeenCalledWith(
      "selectedNodeIds",
      ["node1", "node2"],
    );

    rerender({ selectedNodeIds: [] });
    expect(awareness.setLocalStateField).toHaveBeenCalledWith(
      "selectedNodeIds",
      null,
    );
  });
  it("handleParamFieldFocus sets focusedParamField field", () => {
    const { result } = renderHook(() =>
      useAwarenessPresence({
        yAwareness: awareness,
        selectedNodeIds: [],
        openNode: undefined,
      }),
    );
    act(() => {
      result.current.handleParamFieldFocus("field123");
    });
    expect(awareness.setLocalStateField).toHaveBeenCalledWith(
      "focusedParamField",
      "field123",
    );
    act(() => {
      result.current.handleParamFieldFocus(null);
    });
    expect(awareness.setLocalStateField).toHaveBeenCalledWith(
      "focusedParamField",
      null,
    );
  });

  it("syncs openNode prop to awareness field and clears focusedParamField when closed", () => {
    const { rerender } = renderHook(
      ({ openNode }: { openNode: Node | undefined }) =>
        useAwarenessPresence({
          yAwareness: awareness,
          selectedNodeIds: [],
          openNode,
        }),
      {
        initialProps: {
          openNode: {
            id: "node-1",
            type: "batch",
            position: { x: 0, y: 0 },
            measured: { width: 0, height: 0 },
            data: {
              officialName: "officialName",
              inputs: ["input1"],
              outputs: ["output1"],
              params: {},
              customizations: {},
              isCollapsed: true,
              isDisabled: false,
              pseudoInputs: [],
              pseudoOutputs: [],
            },
            style: { width: 0, height: 0 },
          },
        },
      },
    );
    expect(awareness.setLocalStateField).toHaveBeenCalledWith(
      "openNodeId",
      "node-1",
    );

    rerender({ openNode: undefined });
    expect(awareness.setLocalStateField).toHaveBeenCalledWith(
      "openNodeId",
      null,
    );
    expect(awareness.setLocalStateField).toHaveBeenCalledWith(
      "focusedParamField",
      null,
    );
  });
});
