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

    expect(result.current.getVariablesToSave()).toEqual([]);
  });

  test("clearing a saved override does not restore it", () => {
    const { result } = renderHook(() =>
      useTriggerWorkflowVariables([
        { key: "testnum", type: "number", value: 7 },
      ]),
    );
    act(() =>
      result.current.loadDeploymentVariables(
        deployment([param("testnum", "number", 5)]),
        true,
      ),
    );
    expect(result.current.workflowVariablesObject).toEqual({ testnum: 7 });
    act(() =>
      result.current.handleVariablesConfirm([
        { name: "testnum", defaultValue: null },
      ]),
    );

    expect(result.current.getVariablesToSave()).toEqual([]);
  });
});

describe("useTriggerWorkflowVariables resetting a value", () => {
  test("resetting a saved override to the deployment default clears it", () => {
    const { result } = renderHook(() =>
      useTriggerWorkflowVariables([
        { key: "testnum", type: "number", value: 7 },
      ]),
    );
    act(() =>
      result.current.loadDeploymentVariables(
        deployment([param("testnum", "number", 5)]),
        true,
      ),
    );
    act(() =>
      result.current.handleVariablesConfirm([
        { name: "testnum", defaultValue: 5 },
      ]),
    );

    expect(result.current.getVariablesToSave()).toEqual([]);
  });
});

describe("useTriggerWorkflowVariables fetching a deployment file", () => {
  const fileDeployment = (workflowUrl: string): Deployment => ({
    ...deployment([]),
    workflowUrl,
  });
  const jsonResponse = (body: unknown) =>
    ({ ok: true, json: async () => body }) as Response;

  test("is loading until the fetch settles, with the previous deployment cleared", async () => {
    let resolveFetch!: (r: Response) => void;
    const fetchSpy = vi
      .spyOn(globalThis, "fetch")
      .mockImplementationOnce(
        () => new Promise<Response>((r) => (resolveFetch = r)),
      );
    const { result } = renderHook(() => useTriggerWorkflowVariables());
    act(() =>
      result.current.loadDeploymentVariables(
        deployment([param("b", "number", 5)]),
      ),
    );

    let pending!: Promise<void> | undefined;
    act(() => {
      pending = result.current.loadDeploymentVariables(fileDeployment("a"));
    });
    expect(result.current.isLoadingVariables).toBe(true);
    expect(result.current.workflowVariablesObject).toBeUndefined();
    expect(result.current.deploymentDefaultVariables).toBeUndefined();
    expect(result.current.pendingWorkflowData).toBeNull();

    await act(async () => {
      resolveFetch(jsonResponse({ name: "A", with: { a: 1 } }));
      await pending;
    });
    expect(result.current.isLoadingVariables).toBe(false);
    expect(result.current.workflowVariablesObject).toEqual({ a: 1 });
    fetchSpy.mockRestore();
  });

  test("is not loading once the fetch fails", async () => {
    const fetchSpy = vi
      .spyOn(globalThis, "fetch")
      .mockRejectedValueOnce(new Error("network"));
    const errorSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    const { result } = renderHook(() => useTriggerWorkflowVariables());

    await act(() =>
      result.current.loadDeploymentVariables(fileDeployment("a")),
    );

    expect(result.current.isLoadingVariables).toBe(false);
    fetchSpy.mockRestore();
    errorSpy.mockRestore();
  });

  test("a failed fetch drops values left from the previous deployment", async () => {
    const fetchSpy = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValueOnce(jsonResponse({ name: "A", with: { a: 1 } }))
      .mockResolvedValueOnce({ ok: false } as Response);
    const { result } = renderHook(() => useTriggerWorkflowVariables());

    await act(() =>
      result.current.loadDeploymentVariables(fileDeployment("a")),
    );
    act(() =>
      result.current.handleVariablesConfirm([{ name: "a", defaultValue: 2 }]),
    );
    await act(() =>
      result.current.loadDeploymentVariables(fileDeployment("b")),
    );

    expect(result.current.workflowVariablesObject).toBeUndefined();
    expect(result.current.getVariablesToSave()).toBeUndefined();
    fetchSpy.mockRestore();
  });

  test("a failed fetch keeps the trigger's own values", async () => {
    const fetchSpy = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValueOnce({ ok: false } as Response);
    const { result } = renderHook(() =>
      useTriggerWorkflowVariables([{ key: "a", type: "number", value: 9 }]),
    );

    await act(() =>
      result.current.loadDeploymentVariables(fileDeployment("a"), true),
    );

    expect(result.current.workflowVariablesObject).toEqual({ a: 9 });
    fetchSpy.mockRestore();
  });

  test("a fetch superseded by a newer selection is discarded", async () => {
    let resolveFirst!: (r: Response) => void;
    const fetchSpy = vi
      .spyOn(globalThis, "fetch")
      .mockImplementationOnce(
        () => new Promise<Response>((r) => (resolveFirst = r)),
      );
    const { result } = renderHook(() => useTriggerWorkflowVariables());

    let first!: Promise<void> | undefined;
    act(() => {
      first = result.current.loadDeploymentVariables(fileDeployment("a"));
    });
    act(() =>
      result.current.loadDeploymentVariables(
        deployment([param("b", "number", 5)]),
      ),
    );
    await act(async () => {
      resolveFirst(jsonResponse({ name: "A", with: { a: 1 } }));
      await first;
    });

    expect(result.current.workflowVariablesObject).toEqual({ b: 5 });
    expect(result.current.deploymentDefaultVariables).toEqual({ b: 5 });
    expect(result.current.pendingWorkflowData?.workflowName).toBe("Project");
    fetchSpy.mockRestore();
  });
});
