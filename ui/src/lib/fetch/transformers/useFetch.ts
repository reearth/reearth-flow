import { useQuery } from "@tanstack/react-query";

import { config } from "@flow/config";
import type { Action, Segregated } from "@flow/types";

export type FetchResponse = {
  json: <T = unknown>() => Promise<T>;
} & Response;

enum ActionFetchKeys {
  actions = "actions",
  segregated = "segregated",
}

const BASE_URL = config().api;

const CHANGE_NAMES: Record<string, string> = {
  processor: "transformer",
  sink: "writer",
  source: "reader",
};

const actionResponse = <T extends Action | Action[] | Segregated>(
  response: T,
): T => {
  if (Array.isArray(response)) {
    return response.map((tr) => processAction(tr)) as T;
  } else if (typeof response?.name === "string") {
    return processAction(response as Action) as T;
  }

  // This is because TS doesn't have a way to differentiate between either A or B when writing A | B
  // Details: https://stackoverflow.com/questions/46370222/why-does-a-b-allow-a-combination-of-both-and-how-can-i-prevent-it
  const segregated: Segregated = response as Segregated;
  return Object.keys(segregated).reduce((obj, rootKey) => {
    obj[rootKey] = Object.keys(segregated[rootKey]).reduce(
      (obj: Record<string, Action[] | undefined>, key) => {
        const actions = segregated[rootKey][key]?.map((a) => processAction(a));
        if (CHANGE_NAMES[key]) {
          obj[CHANGE_NAMES[key]] = actions;
        } else {
          obj[key] = actions;
        }
        return obj;
      },
      {},
    );
    return obj;
  }, {} as Segregated) as T;

  function processAction(action: Action) {
    return {
      ...action,
      type: CHANGE_NAMES[action.type] ? CHANGE_NAMES[action.type] : action.type,
    };
  }
};

export const fetcher = async <T extends Action[] | Segregated | Action>(
  url: string,
  signal?: AbortSignal,
): Promise<T> => {
  const response = await fetch(url, { signal });

  if (!response.ok) {
    throw new Error("response not ok");
  }
  const status = response.status;
  if (status != 200) {
    throw new Error(`status not 200. received ${status}`);
  }
  const data = await response.json();
  return actionResponse(data);
};

export const useFetch = () => {
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
        fetcher<Segregated>(
          `${BASE_URL}/actions/${ActionFetchKeys.segregated}`,
          signal,
        ),
      staleTime: Infinity,
    });

  return {
    useGetActionsFetch,
    useGetActionsByIdFetch,
    useGetActionsSegregatedFetch,
  };
};
