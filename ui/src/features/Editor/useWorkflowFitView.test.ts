import { act, renderHook } from "@testing-library/react";

import useWorkflowFitView from "./useWorkflowFitView";

const fitView = vi.fn();
let state: { nodeLookup: Map<string, any> };

vi.mock("@xyflow/react", () => ({
  useReactFlow: () => ({ fitView }),
  useStoreApi: () => ({ getState: () => state }),
  useStore: (selector: (s: typeof state) => unknown) => selector(state),
}));

const nodes = (...sizes: [number, number][]) => ({
  nodeLookup: new Map(
    sizes.map(([width, height], i) => [
      `n${i}`,
      { measured: { width, height } },
    ]),
  ),
});

describe("useWorkflowFitView", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    fitView.mockClear();
  });
  afterEach(() => vi.useRealTimers());

  it("fits straight away when the nodes already have a size", () => {
    state = nodes([150, 25]);
    renderHook(() => useWorkflowFitView("main"));
    expect(fitView).toHaveBeenCalledTimes(1);
  });

  it("waits for a never-drawn workflow's nodes to be measured", () => {
    state = nodes([150, 25]);
    const { rerender } = renderHook(({ id }) => useWorkflowFitView(id), {
      initialProps: { id: "main" },
    });
    fitView.mockClear();

    // A freshly created subworkflow: its routers are stored 0×0.
    state = nodes([0, 0], [0, 0]);
    rerender({ id: "sub" });
    expect(fitView).not.toHaveBeenCalled();

    state = nodes([150, 25], [150, 25]);
    rerender({ id: "sub" });
    expect(fitView).toHaveBeenCalledTimes(1);

    act(() => vi.advanceTimersByTime(1000));
    expect(fitView).toHaveBeenCalledTimes(1);
  });

  it("fits anyway if a node never reports a size", () => {
    state = nodes([0, 0]);
    renderHook(() => useWorkflowFitView("sub"));
    expect(fitView).not.toHaveBeenCalled();

    act(() => vi.advanceTimersByTime(500));
    expect(fitView).toHaveBeenCalledTimes(1);
  });
});
