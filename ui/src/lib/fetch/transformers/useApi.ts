import { useMemo } from "react";

import {
  Action,
  GetAction,
  GetActions,
  GetActionsSegregated,
} from "@flow/types";

import { useFetch } from "./useFetch";

const filterActions = (
  actions: Action[],
  filter?: {
    isMainWorkflow: boolean;
    searchTerm?: string;
  },
) => {
  if (!actions) return [];

  let result = [...actions];

  if (filter?.isMainWorkflow) {
    result = result.filter(
      (action) => !action.name.toLowerCase().includes("router"),
    );
  }

  if (filter?.searchTerm) {
    result = result.filter((action) =>
      Object.values(action).some((value) => {
        const strValue = Array.isArray(value)
          ? value.join(" ")
          : typeof value === "string"
            ? value
            : String(value);
        return strValue
          .toLowerCase()
          .includes(filter.searchTerm?.toLowerCase() ?? "");
      }),
    );
  }

  return result;
};

const filterActionsByPredicate = (
  obj: Record<string, Action[] | undefined>,
  predicate: (action: Action) => boolean,
  removeEmptyArrays = false,
) => {
  const entries = Object.entries(obj).map(([key, actions]) => [
    key,
    actions?.filter(predicate),
  ]);

  return Object.fromEntries(
    removeEmptyArrays
      ? entries.filter(([_, actions]) => actions && actions.length > 0)
      : entries,
  );
};

const filterBySearchTerm = (action: Action, searchTerm?: string) => {
  return Object.values(action).some((value) => {
    const strValue = Array.isArray(value)
      ? value.join(" ")
      : typeof value === "string"
        ? value
        : String(value);
    return strValue.toLowerCase().includes(searchTerm?.toLowerCase() ?? "");
  });
};

export const useAction = (lang: string) => {
  const {
    useGetActionsFetch,
    useGetActionsByIdFetch,
    useGetActionsSegregatedFetch,
  } = useFetch();

  const useGetActions = (filter?: {
    isMainWorkflow: boolean;
    searchTerm?: string;
  }): GetActions => {
    const { data, ...rest } = useGetActionsFetch(lang);

    const filteredData = useMemo(() => {
      if (!data) return data;
      return filterActions(data, filter);
    }, [data, filter]);

    return {
      actions: filteredData,
      ...rest,
    };
  };

  const useGetActionById = (id: string): GetAction => {
    const { data, ...rest } = useGetActionsByIdFetch(id, lang);
    return {
      action: data,
      ...rest,
    };
  };

  const useGetActionsSegregated = (filter?: {
    isMainWorkflow: boolean;
    searchTerm?: string;
    type?: string;
  }): GetActionsSegregated => {
    const { data, ...rest } = useGetActionsSegregatedFetch(lang);

    const filteredData = useMemo(() => {
      if (!data) return data;

      let result = { ...data };

      if (filter?.isMainWorkflow) {
        const filterOutRouter = (action: Action) =>
          !action.name.toLowerCase().includes("router");

        result = {
          byCategory: filterActionsByPredicate(
            result.byCategory,
            filterOutRouter,
          ),
          byType: filterActionsByPredicate(result.byType, filterOutRouter),
        };
      }

      if (filter?.searchTerm) {
        result = {
          byCategory: filterActionsByPredicate(
            result.byCategory,
            (action) => filterBySearchTerm(action, filter.searchTerm),
            true,
          ),
          byType: filterActionsByPredicate(
            result.byType,
            (action) => filterBySearchTerm(action, filter.searchTerm),
            true,
          ),
        };
      }

      if (filter?.type && result.byType) {
        return {
          ...result,
          byType: {
            [filter.type]: result.byType[filter.type],
          },
        };
      }

      return result;
    }, [data, filter?.isMainWorkflow, filter?.searchTerm, filter?.type]);

    return {
      actions: filteredData,
      ...rest,
    };
  };

  return {
    useGetActions,
    useGetActionById,
    useGetActionsSegregated,
  };
};
