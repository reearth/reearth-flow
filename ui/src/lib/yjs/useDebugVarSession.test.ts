import { renderHook, act } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { Doc } from "yjs";

import type { AnyWorkflowVariable } from "@flow/types";

import useDebugVarSession from "./useDebugVarSession";

const base = [
  { id: "v1", name: "city", defaultValue: "tokyo" },
  { id: "v2", name: "limit", defaultValue: 10 },
] as unknown as AnyWorkflowVariable[];

const render = (yDoc: Doc | null, baseVariables = base) =>
  renderHook(() => useDebugVarSession({ yDoc, baseVariables }));

describe("useDebugVarSession", () => {
  it("passes base variables through when nothing is staged", () => {
    const { result } = render(new Doc());
    expect(result.current.variables).toEqual(base);
    expect(result.current.hasOverrides).toBe(false);
  });

  it("applies a staged value over the base", () => {
    const { result } = render(new Doc());

    act(() => result.current.setVariableValue("v1", "osaka"));

    expect(result.current.variables[0].defaultValue).toBe("osaka");
    expect(result.current.variables[1].defaultValue).toBe(10);
    expect(result.current.hasOverrides).toBe(true);
  });

  it("shares staged values between clients on the same doc", () => {
    // Two hooks over one doc stand in for two users in the same session.
    const yDoc = new Doc();
    const a = render(yDoc);
    const b = render(yDoc);

    act(() => a.result.current.setVariableValue("v2", 99));

    expect(b.result.current.variables[1].defaultValue).toBe(99);
  });

  it("clears every staged value", () => {
    const { result } = render(new Doc());

    act(() => result.current.setVariableValue("v1", "kyoto"));
    act(() => result.current.clearSession());

    expect(result.current.variables).toEqual(base);
    expect(result.current.hasOverrides).toBe(false);
  });

  it("keeps staged values keyed by id when the base list reorders", () => {
    const yDoc = new Doc();
    const { result, rerender } = renderHook(
      (props: { baseVariables: AnyWorkflowVariable[] }) =>
        useDebugVarSession({ yDoc, baseVariables: props.baseVariables }),
      { initialProps: { baseVariables: base } },
    );

    act(() => result.current.setVariableValue("v2", 42));
    rerender({ baseVariables: [base[1], base[0]] });

    expect(result.current.variables[0].id).toBe("v2");
    expect(result.current.variables[0].defaultValue).toBe(42);
    expect(result.current.variables[1].defaultValue).toBe("tokyo");
  });

  it("is inert without a doc", () => {
    const { result } = render(null);
    expect(result.current.variables).toEqual(base);
    act(() => result.current.setVariableValue("v1", "nope"));
    expect(result.current.variables[0].defaultValue).toBe("tokyo");
  });
});
