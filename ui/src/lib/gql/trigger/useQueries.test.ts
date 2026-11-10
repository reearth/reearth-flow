import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook } from "@testing-library/react";
import { createElement } from "react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";

import { toGqlVariableInput } from "../convert";

import { TriggerQueryKeys, useQueries } from "./useQueries";

const UpdateTrigger = vi.fn();
vi.mock("../provider", () => ({
  useGraphQLContext: () => ({ UpdateTrigger }),
}));

describe("toGqlVariableInput", () => {
  it("sends a known type as its server type", () => {
    expect(
      toGqlVariableInput([{ key: "area", type: "choice", value: "tokyo" }]),
    ).toEqual([{ key: "area", type: "CHOICE", value: "tokyo" }]);
  });

  it("keeps a variable whose type the server has no counterpart for, as text", () => {
    // A URL value is inferred as web_connection, which the server lacks.
    expect(
      toGqlVariableInput([
        {
          key: "source",
          type: "web_connection",
          value: "https://example.com/data.zip",
        },
        { key: "apiKey", type: "password", value: "secret" },
      ]),
    ).toEqual([
      { key: "source", type: "TEXT", value: "https://example.com/data.zip" },
      { key: "apiKey", type: "TEXT", value: "secret" },
    ]);
  });
});

describe("updateTriggerMutation", () => {
  it("refetches the triggers when an update fails, as its edit may have been saved", async () => {
    UpdateTrigger.mockRejectedValue(
      new Error(
        "the trigger was saved, but its schedule could not be updated; save it again to retry",
      ),
    );
    const queryClient = new QueryClient();
    const invalidate = vi.spyOn(queryClient, "invalidateQueries");
    const wrapper = ({ children }: { children: ReactNode }) =>
      createElement(QueryClientProvider, { client: queryClient }, children);
    const { result } = renderHook(() => useQueries(), { wrapper });

    await act(async () => {
      await result.current.updateTriggerMutation
        .mutateAsync({ triggerId: "t" })
        .catch(() => undefined);
    });

    expect(invalidate).toHaveBeenCalledWith({
      queryKey: [TriggerQueryKeys.GetTriggers],
    });
  });
});
