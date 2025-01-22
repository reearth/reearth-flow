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
  const BASE_URL = config().api;
  const useGetActionsFetch = (lang: string) =>
    useQuery({
      queryKey: [ActionFetchKeys.actions, lang],
      queryFn: async ({ signal }: { signal: AbortSignal }) => {
        return fetcher<Action[]>(`${BASE_URL}/actions?lang=${lang}`, signal);
      },
      staleTime: Infinity,
    });

  const useGetActionsByIdFetch = (actionId: string, lang: string) =>
    useQuery({
      queryKey: [ActionFetchKeys.actions, actionId, lang],
      queryFn: async ({ signal }: { signal: AbortSignal }) => {
        return fetcher<Action>(
          `${BASE_URL}/actions/${actionId}?lang=${lang}`,
          signal,
        );
      },
      staleTime: Infinity,
    });

  const useGetActionsSegregatedFetch = (lang: string) =>
    useQuery({
      queryKey: [ActionFetchKeys.actions, ActionFetchKeys.segregated, lang],
      queryFn: async ({ signal }: { signal: AbortSignal }) => {
        return fetcher<Segregated>(
          `${BASE_URL}/actions/${ActionFetchKeys.segregated}?lang=${lang}`,
          signal,
        );
      },
      staleTime: Infinity,
    });

  return {
    useGetActionsFetch,
    useGetActionsByIdFetch,
    useGetActionsSegregatedFetch,
  };
};
