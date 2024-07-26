import { useQuery } from "@tanstack/react-query";

import { config } from "@flow/config";
import { Action, Segregated } from "@flow/types";

export type FetchResponse = {
  json: <T = unknown>() => Promise<T>;
} & Response;

enum ActionFetchKeys {
  actions = "actions",
  segregated = "segregated",
}

const BASE_URL = config().api;

export const useFetch = () => {
  const fetcher = async <T>(url: string, signal: AbortSignal): Promise<T> => {
    const response = await fetch(url, { signal });

    if (!response.ok) {
      throw new Error("response not ok");
    }
    const status = response.status;
    if (status != 200) {
      throw new Error(`status not 200. received ${status}`);
    }
    const data = response.json();
    return data;
  };

  const useGetActionsFetch = () =>
    useQuery({
      queryKey: [ActionFetchKeys.actions],
      queryFn: async ({ signal }: { signal: AbortSignal }) =>
        fetcher<Action[]>(`${BASE_URL}/actions`, signal),
      staleTime: Infinity,
    });

  const useGetActionsByIdFetch = (actionId: string) =>
    useQuery({
      queryKey: [ActionFetchKeys.actions, actionId],
      queryFn: async ({ signal }: { signal: AbortSignal }) =>
        fetcher<Action>(`${BASE_URL}/actions/${actionId}`, signal),
      staleTime: Infinity,
    });

  const useGetActionsSegregatedFetch = () =>
    useQuery({
      queryKey: [ActionFetchKeys.actions, ActionFetchKeys.segregated],
      queryFn: async ({ signal }: { signal: AbortSignal }) =>
        fetcher<Segregated>(`${BASE_URL}/actions/${ActionFetchKeys.segregated}`, signal),
      staleTime: Infinity,
    });

  return {
    useGetActionsFetch,
    useGetActionsByIdFetch,
    useGetActionsSegregatedFetch,
  };
};
