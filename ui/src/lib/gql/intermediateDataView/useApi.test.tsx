import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { GraphQLError } from "graphql";
import { ClientError } from "graphql-request";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, test, vi } from "vitest";

import type { IntermediateDataViewFragment } from "@flow/lib/gql/__gen__/plugins/graphql-request";
import type {
  IntermediateDataViewErrorKind,
  IntermediateDataViewRequest,
} from "@flow/types";

import { IntermediateDataViewError } from "./errors";
import { useIntermediateDataView } from "./useApi";
import { MAX_RENDERS_IN_FLIGHT } from "./useQueries";

const deferred = <T,>() => {
  let resolve: (value: T) => void = () => {};
  let reject: (reason: unknown) => void = () => {};
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
};

const viewFragment = (
  overrides: Partial<IntermediateDataViewFragment> = {},
): IntermediateDataViewFragment => ({
  id: "tiles-all-abc",
  jobId: "job1",
  fileId: "n1.default",
  shape: "TILES",
  status: "READY",
  format: "VECTOR_TILES",
  entryPointUrl: "https://api.example.com/artifacts/v/tilejson.json",
  selectedFeatures: 8,
  renderedFeatures: 8,
  sizeLimitedFeatures: 0,
  sizeLimitedTiles: 0,
  error: null,
  ...overrides,
});

type TilesResult = {
  renderIntermediateDataTilesView: { view: IntermediateDataViewFragment };
};
type FeatureResult = {
  renderIntermediateDataFeatureView: { view: IntermediateDataViewFragment };
};

const tiles = (view = viewFragment()): TilesResult => ({
  renderIntermediateDataTilesView: { view },
});
const feature = (view = viewFragment({ shape: "GLTF" })): FeatureResult => ({
  renderIntermediateDataFeatureView: { view },
});

const sdk = {
  RenderIntermediateDataTilesView: vi.fn<() => Promise<TilesResult>>(),
  RenderIntermediateDataFeatureView: vi.fn<() => Promise<FeatureResult>>(),
};

vi.mock("@flow/lib/gql", () => ({ useGraphQLContext: () => sdk }));

const port: IntermediateDataViewRequest = {
  jobId: "job1",
  fileId: "n1.default",
};
const row = (n: number): IntermediateDataViewRequest => ({ ...port, row: n });

const setup = () => {
  const queryClient = new QueryClient();
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  return renderHook(() => useIntermediateDataView(), { wrapper });
};

beforeEach(() => {
  sdk.RenderIntermediateDataTilesView.mockReset();
  sdk.RenderIntermediateDataFeatureView.mockReset();
});

describe("requestView", () => {
  test("renders a whole port as tiles", async () => {
    sdk.RenderIntermediateDataTilesView.mockResolvedValue(tiles());
    const { result } = setup();

    const view = await result.current.requestView(port);

    expect(sdk.RenderIntermediateDataTilesView).toHaveBeenCalledWith({
      input: { jobId: "job1", fileId: "n1.default" },
    });
    expect(view).toMatchObject({ status: "ready", format: "vectorTiles" });
  });

  test("renders a single row as a 3D model", async () => {
    sdk.RenderIntermediateDataFeatureView.mockResolvedValue(feature());
    const { result } = setup();

    await result.current.requestView(row(3));

    expect(sdk.RenderIntermediateDataFeatureView).toHaveBeenCalledWith({
      input: { jobId: "job1", fileId: "n1.default", row: 3 },
    });
    expect(sdk.RenderIntermediateDataTilesView).not.toHaveBeenCalled();
  });

  test("joins a render already running for the same view", async () => {
    const pending = deferred<TilesResult>();
    sdk.RenderIntermediateDataTilesView.mockReturnValue(pending.promise);
    const { result } = setup();

    const first = result.current.requestView(port);
    const second = result.current.requestView(port);
    pending.resolve(tiles());

    await expect(first).resolves.toMatchObject({ status: "ready" });
    await expect(second).resolves.toMatchObject({ status: "ready" });
    expect(sdk.RenderIntermediateDataTilesView).toHaveBeenCalledTimes(1);
  });

  test("refuses a third render while two are running, but still joins a running one", async () => {
    const renders = [deferred<FeatureResult>(), deferred<FeatureResult>()];
    sdk.RenderIntermediateDataFeatureView.mockReturnValueOnce(
      renders[0].promise,
    ).mockReturnValueOnce(renders[1].promise);
    const { result } = setup();

    const first = result.current.requestView(row(0));
    const second = result.current.requestView(row(1));

    await expect(result.current.requestView(row(2))).rejects.toMatchObject({
      kind: "tooManyRenders",
    });
    const joined = result.current.requestView(row(0));

    renders[0].resolve(feature());
    renders[1].resolve(feature());
    await Promise.all([first, second, joined]);
    expect(sdk.RenderIntermediateDataFeatureView).toHaveBeenCalledTimes(2);

    // With both finished there is room again.
    sdk.RenderIntermediateDataFeatureView.mockResolvedValue(feature());
    await expect(result.current.requestView(row(2))).resolves.toBeDefined();
  });

  test("answers a finished view from the cache", async () => {
    sdk.RenderIntermediateDataTilesView.mockResolvedValue(tiles());
    const { result } = setup();

    await result.current.requestView(port);
    await result.current.requestView(port);

    expect(sdk.RenderIntermediateDataTilesView).toHaveBeenCalledTimes(1);
  });

  test("keeps a view that has nothing to show, since asking again cannot change it", async () => {
    sdk.RenderIntermediateDataTilesView.mockResolvedValue(
      tiles(
        viewFragment({ status: "EMPTY", format: null, entryPointUrl: null }),
      ),
    );
    const { result } = setup();

    await result.current.requestView(port);
    await result.current.requestView(port);

    expect(sdk.RenderIntermediateDataTilesView).toHaveBeenCalledTimes(1);
  });

  test("asks again for a failed view, which the server renders afresh", async () => {
    sdk.RenderIntermediateDataTilesView.mockResolvedValueOnce(
      tiles(
        viewFragment({ status: "FAILED", format: null, entryPointUrl: null }),
      ),
    ).mockResolvedValueOnce(tiles());
    const { result } = setup();

    await expect(result.current.requestView(port)).resolves.toMatchObject({
      status: "failed",
    });
    await expect(result.current.requestView(port)).resolves.toMatchObject({
      status: "ready",
    });
    expect(sdk.RenderIntermediateDataTilesView).toHaveBeenCalledTimes(2);
  });

  test("classifies a request the server refused, and does not retry it", async () => {
    sdk.RenderIntermediateDataTilesView.mockRejectedValue(
      new ClientError(
        {
          errors: [
            new GraphQLError("the view did not finish rendering in time"),
          ],
          status: 200,
          headers: new Headers(),
          body: "",
        },
        { query: "" },
      ),
    );
    const { result } = setup();

    await expect(result.current.requestView(port)).rejects.toMatchObject({
      kind: "timedOut",
    });
    expect(sdk.RenderIntermediateDataTilesView).toHaveBeenCalledTimes(1);
  });
});

describe("errorMessage", () => {
  test("has a message for every reason a view could not be requested", () => {
    const { result } = setup();
    const kinds: IntermediateDataViewErrorKind[] = [
      "timedOut",
      "unavailable",
      "jobNotFinished",
      "dataNotFound",
      "permissionDenied",
      "tooManyRenders",
      "unknown",
    ];

    const messages = kinds.map((kind) =>
      result.current.errorMessage(new IntermediateDataViewError(kind)),
    );

    expect(messages.every((m) => m.length > 0)).toBe(true);
    expect(new Set(messages).size).toBe(kinds.length);
  });

  test("says how many renders may run at once", () => {
    const { result } = setup();

    expect(
      result.current.errorMessage(
        new IntermediateDataViewError("tooManyRenders"),
      ),
    ).toContain(String(MAX_RENDERS_IN_FLIGHT));
  });
});

describe("useView", () => {
  const setupWithView = (request?: IntermediateDataViewRequest) => {
    const queryClient = new QueryClient();
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
    return renderHook(
      () => {
        const { useView, requestView } = useIntermediateDataView();
        return { state: useView(request), requestView };
      },
      { wrapper },
    );
  };

  test("never starts a render by itself", () => {
    const { result } = setupWithView(port);

    expect(result.current.state).toMatchObject({
      view: undefined,
      isRendering: false,
    });
    expect(sdk.RenderIntermediateDataTilesView).not.toHaveBeenCalled();
  });

  test("reports a running render, then its view", async () => {
    const pending = deferred<TilesResult>();
    sdk.RenderIntermediateDataTilesView.mockReturnValue(pending.promise);
    const { result } = setupWithView(port);

    let request: Promise<unknown> = Promise.resolve();
    act(() => {
      request = result.current.requestView(port);
    });
    await waitFor(() => expect(result.current.state.isRendering).toBe(true));
    expect(result.current.state.renderStartedAt).toEqual(expect.any(Number));

    pending.resolve(tiles());
    await act(() => request);

    await waitFor(() =>
      expect(result.current.state).toMatchObject({
        isRendering: false,
        renderStartedAt: undefined,
        view: { status: "ready" },
      }),
    );
  });

  test("keeps a refused request's reason", async () => {
    sdk.RenderIntermediateDataTilesView.mockRejectedValue(
      new ClientError(
        {
          errors: [new GraphQLError("job has not finished: job1 is running")],
          status: 200,
          headers: new Headers(),
          body: "",
        },
        { query: "" },
      ),
    );
    const { result } = setupWithView(port);

    await act(() =>
      result.current.requestView(port).catch(() => {
        // Read back through useView below.
      }),
    );

    await waitFor(() =>
      expect(result.current.state.error?.kind).toBe("jobNotFinished"),
    );
  });

  test("reports nothing without a request", () => {
    const { result } = setupWithView(undefined);

    expect(result.current.state).toMatchObject({
      view: undefined,
      error: undefined,
      isRendering: false,
    });
  });
});
