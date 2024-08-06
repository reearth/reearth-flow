import { useQuery } from "@tanstack/react-query";

import { config } from "@flow/config";
import { Transformer, Segregated } from "@flow/types";

export type FetchResponse = {
  json: <T = unknown>() => Promise<T>;
} & Response;

enum ActionFetchKeys {
  actions = "actions",
  segregated = "segregated",
}

const BASE_URL = config().api;

export const useFetch = () => {
  const transformResponse = <
    T extends Transformer | Transformer[] | Segregated,
  >(
    response: T
  ): T => {
    const CHANGE_NAMES: Record<string, string> = {
      processor: "Transformer",
      sink: "Writer",
      source: "Reader",
    };

    if (Array.isArray(response)) {
      return response.map((tr) => processTransformer(tr)) as T;
    } else if (typeof response?.name === "string") {
      return processTransformer(response as Transformer) as T;
    }

    // This is because TS doesn't have a way to differentiate between either A or B when writing A | B
    // Details: https://stackoverflow.com/questions/46370222/why-does-a-b-allow-a-combination-of-both-and-how-can-i-prevent-it
    const segregated: Segregated = response as Segregated;
    return Object.keys(segregated).reduce((obj, rootKey) => {
      obj[rootKey] = Object.keys(segregated[rootKey]).reduce(
        (obj: Record<string, Transformer[] | undefined>, key) => {
          const transformers = segregated[rootKey][key]?.map((a) =>
            processTransformer(a)
          );
          if (CHANGE_NAMES[key]) {
            obj[CHANGE_NAMES[key]] = transformers;
          } else {
            obj[key] = transformers;
          }
          return obj;
        },
        {}
      );
      return obj;
    }, {} as Segregated) as T;

    function processTransformer(transformer: Transformer) {
      return {
        ...transformer,
        type: CHANGE_NAMES[transformer.type]
          ? CHANGE_NAMES[transformer.type]
          : transformer.type,
      };
    }
  };

  const fetcher = async <T extends Transformer[] | Segregated | Transformer>(
    url: string,
    signal: AbortSignal
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
    return transformResponse(data);
  };

  const useGetTransformersFetch = () =>
    useQuery({
      queryKey: [ActionFetchKeys.actions],
      queryFn: async ({ signal }: { signal: AbortSignal }) =>
        fetcher<Transformer[]>(`${BASE_URL}/actions`, signal),
      staleTime: Infinity,
    });

  const useGetTransformersByIdFetch = (actionId: string) =>
    useQuery({
      queryKey: [ActionFetchKeys.actions, actionId],
      queryFn: async ({ signal }: { signal: AbortSignal }) =>
        fetcher<Transformer>(`${BASE_URL}/actions/${actionId}`, signal),
      staleTime: Infinity,
    });

  const useGetTransformersSegregatedFetch = () =>
    useQuery({
      queryKey: [ActionFetchKeys.actions, ActionFetchKeys.segregated],
      queryFn: async ({ signal }: { signal: AbortSignal }) =>
        fetcher<Segregated>(
          `${BASE_URL}/actions/${ActionFetchKeys.segregated}`,
          signal
        ),
      staleTime: Infinity,
    });

  return {
    useGetTransformersFetch,
    useGetTransformersByIdFetch,
    useGetTransformersSegregatedFetch,
  };
};
