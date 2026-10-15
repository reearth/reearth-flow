import {
  queryOptions,
  skipToken,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { useCallback } from "react";

import { useGraphQLContext } from "@flow/lib/gql";
import type {
  IntermediateDataView,
  IntermediateDataViewRequest,
} from "@flow/types";

import { toIntermediateDataView } from "../convert";

import {
  IntermediateDataViewError,
  toIntermediateDataViewError,
} from "./errors";
import { IntermediateDataViewQueryKeys } from "./useApi";

/**
 * Renders this tab may have running at once. Renders run on a small pool of
 * workers that debug runs share, so the limit keeps one user from occupying
 * it; the server's own limit, once it has one, is the authority.
 */
export const MAX_RENDERS_IN_FLIGHT = 2;

// A view is a pure function of its request, so a finished one never goes stale
// and is kept for the session. Thirty minutes matches the table's own cache.
const VIEW_CACHE_TIME = 30 * 60 * 1000;

const viewQueryKey = (request?: IntermediateDataViewRequest) =>
  [
    IntermediateDataViewQueryKeys.View,
    request?.jobId ?? "",
    request?.fileId ?? "",
    request?.row ?? "all",
  ] as const;

// When each running render started, by query key, so an elapsed time survives
// the component that asked for the view unmounting and another mounting.
const renderStartedAt = new Map<string, number>();

const renderView = async (
  graphQLContext: ReturnType<typeof useGraphQLContext>,
  request: IntermediateDataViewRequest,
): Promise<IntermediateDataView> => {
  const startedAtKey = JSON.stringify(viewQueryKey(request));
  renderStartedAt.set(startedAtKey, Date.now());
  try {
    if (!graphQLContext) {
      throw new IntermediateDataViewError(
        "unknown",
        "The API client is not ready",
      );
    }
    const { jobId, fileId, row } = request;
    if (row === undefined) {
      const data = await graphQLContext.RenderIntermediateDataTilesView({
        input: { jobId, fileId },
      });
      return toIntermediateDataView(data.renderIntermediateDataTilesView.view);
    }
    const data = await graphQLContext.RenderIntermediateDataFeatureView({
      input: { jobId, fileId, row },
    });
    return toIntermediateDataView(data.renderIntermediateDataFeatureView.view);
  } catch (err) {
    throw toIntermediateDataViewError(err);
  } finally {
    renderStartedAt.delete(startedAtKey);
  }
};

const viewQueryOptions = (
  graphQLContext: ReturnType<typeof useGraphQLContext>,
  request?: IntermediateDataViewRequest,
) =>
  queryOptions({
    queryKey: viewQueryKey(request),
    queryFn: request ? () => renderView(graphQLContext, request) : skipToken,
    // A retry is another render on a shared worker, so it is only ever the
    // user's decision.
    retry: false,
    staleTime: Infinity,
    gcTime: VIEW_CACHE_TIME,
  });

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  /**
   * Observes a view without ever starting a render. It reports a render
   * already running for the same request, even one started by a component
   * that has since unmounted.
   */
  const useViewQuery = (request?: IntermediateDataViewRequest) =>
    useQuery({ ...viewQueryOptions(graphQLContext, request), enabled: false });

  /**
   * Asks for a view, rendering it if the server has none. Asking for a view
   * that is already rendering joins that render rather than starting another.
   */
  const fetchView = useCallback(
    async (
      request: IntermediateDataViewRequest,
    ): Promise<IntermediateDataView> => {
      const options = viewQueryOptions(graphQLContext, request);
      const state = queryClient.getQueryState(options.queryKey);
      const alreadyRendering = state?.fetchStatus === "fetching";
      if (
        !alreadyRendering &&
        queryClient.isFetching({
          queryKey: [IntermediateDataViewQueryKeys.View],
        }) >= MAX_RENDERS_IN_FLIGHT
      ) {
        throw new IntermediateDataViewError("tooManyRenders");
      }

      // Every status but failed is final for a request, so a cached view is
      // the answer. A failed one is asked again, which the server renders
      // afresh.
      const askAgain = state?.data?.status === "failed";
      return queryClient.fetchQuery({
        ...options,
        staleTime: askAgain ? 0 : options.staleTime,
      });
    },
    [graphQLContext, queryClient],
  );

  return { useViewQuery, fetchView };
};

/** When the render for `request` started, or undefined when none is running. */
export const getRenderStartedAt = (
  request: IntermediateDataViewRequest,
): number | undefined =>
  renderStartedAt.get(JSON.stringify(viewQueryKey(request)));
