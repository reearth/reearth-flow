import { act, renderHook } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";

import type { AnyWorkflowVariable, Deployment } from "@flow/types";

import { useTriggerWorkflowVariables } from "./useTriggerWorkflowVariables";

const param = (
  name: string,
  type: AnyWorkflowVariable["type"],
  defaultValue: any,
  required = false,
): AnyWorkflowVariable => ({
  id: name,
  name,
  type,
  defaultValue,
  required,
  public: true,
});

const deployment = (parameters: AnyWorkflowVariable[]): Deployment => ({
  id: "d1",
  workspaceId: "w1",
  workflowUrl: "https://example.com/workflow.json",
  version: "1",
  createdAt: "",
  updatedAt: "",
  projectName: "Project",
  parameters,
});

describe("useTriggerWorkflowVariables with saved definitions", () => {
  const params = [
    param("city", "text", "Tokyo", true),
    param("apiKey", "text", null, true),
    param("count", "number", 3),
  ];

  test("loads the deployment's variables without fetching its file", () => {
    const fetchSpy = vi.spyOn(globalThis, "fetch");
    const { result } = renderHook(() => useTriggerWorkflowVariables());
    act(() => result.current.loadDeploymentVariables(deployment(params)));

    expect(fetchSpy).not.toHaveBeenCalled();
    expect(result.current.definitions).toBe(params);
    expect(result.current.missingRequired).toEqual(["apiKey"]);
    fetchSpy.mockRestore();
  });

  test("saves only changed values, typed as the deployment defines them", () => {
    const { result } = renderHook(() => useTriggerWorkflowVariables());
    act(() => result.current.loadDeploymentVariables(deployment(params)));
    act(() =>
      result.current.handleVariablesConfirm([
        { name: "city", defaultValue: "Tokyo" },
        { name: "apiKey", defaultValue: "secret" },
        { name: "count", defaultValue: "7" },
      ]),
    );

    expect(result.current.missingRequired).toEqual([]);
    expect(result.current.getVariablesToSave()).toEqual([
      { key: "apiKey", type: "text", value: "secret" },
      { key: "count", type: "number", value: "7" },
    ]);
  });

  test("keeps a trigger's existing values and flags those the deployment dropped", () => {
    const { result } = renderHook(() =>
      useTriggerWorkflowVariables([
        { key: "apiKey", type: "text", value: "k" },
        { key: "oldVar", type: "text", value: "x" },
      ]),
    );
    act(() => result.current.loadDeploymentVariables(deployment(params), true));

    expect(result.current.workflowVariablesObject).toEqual({
      city: "Tokyo",
      apiKey: "k",
      count: 3,
      oldVar: "x",
    });
    expect(result.current.staleVariables).toEqual(["oldVar"]);
    expect(result.current.missingRequired).toEqual([]);
  });
});

describe("useTriggerWorkflowVariables clearing a value", () => {
  test("a cleared field is not saved as an override", () => {
    const { result } = renderHook(() => useTriggerWorkflowVariables());
    act(() =>
      result.current.loadDeploymentVariables(
        deployment([param("testnum", "number", 5)]),
      ),
    );
    act(() =>
      result.current.handleVariablesConfirm([
        { name: "testnum", defaultValue: null },
      ]),
    );

    expect(result.current.getVariablesToSave()).toBeUndefined();
  });
});
