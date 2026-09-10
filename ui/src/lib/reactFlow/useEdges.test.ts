import { act, renderHook } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";

import type { Edge } from "@flow/types";

import useEdges from "./useEdges";

const edge = (
  id: string,
  sourceHandle: string,
  targetHandle: string,
): Edge => ({
  id,
  source: "a",
  target: "b",
  sourceHandle,
  targetHandle,
});

describe("handleConnect", () => {
  test("ignores a connection that already exists", () => {
    const onEdgesAdd = vi.fn();
    const { result } = renderHook(() =>
      useEdges({ edges: [edge("a-b", "features", "features")], onEdgesAdd }),
    );

    act(() =>
      result.current.handleConnect({
        source: "a",
        target: "b",
        sourceHandle: "features",
        targetHandle: "features",
      }),
    );

    expect(onEdgesAdd).not.toHaveBeenCalled();
  });

  test("allows a second connection between different handles", () => {
    const onEdgesAdd = vi.fn();
    const { result } = renderHook(() =>
      useEdges({ edges: [edge("a-b", "features", "features")], onEdgesAdd }),
    );

    act(() =>
      result.current.handleConnect({
        source: "a",
        target: "b",
        sourceHandle: "rejected",
        targetHandle: "features",
      }),
    );

    expect(onEdgesAdd).toHaveBeenCalledTimes(1);
  });
});
