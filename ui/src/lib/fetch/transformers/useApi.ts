import { useMemo } from "react";

import {
  Action,
  GetAction,
  GetActions,
  GetActionsSegregated,
  Node,
} from "@flow/types";

import { useFetch } from "./useFetch";

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
    nodes?: Node[];
  }): GetActionsSegregated => {
    const { data, ...rest } = useGetActionsSegregatedFetch(lang);

    const filteredData = useMemo(() => {
      if (!data) return data;

      let result = { ...data };

      result = {
        byCategory: filterActionsByPredicate(
          result.byCategory,
          (action) => combinedFilter(action, filter),
          !!filter?.searchTerm,
        ),
        byType: filterActionsByPredicate(
          result.byType,
          (action) => combinedFilter(action, filter),
          !!filter?.searchTerm,
        ),
      };

      if (filter?.type && result.byType) {
        return {
          ...result,
          byType: {
            [filter.type]: result.byType[filter.type],
          },
        };
      }

      return result;
    }, [data, filter]);

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

const combinedFilter = (
  action: Action,
  filter?: {
    isMainWorkflow: boolean;
    searchTerm?: string;
    nodes?: Node[];
  },
) => {
  if (filter?.isMainWorkflow) {
    if (action.name.toLowerCase().includes("router")) {
      return false;
    }
  } else {
    if (action.type === "reader" || action.type === "writer") {
      return false;
    }
  }

  if (filter?.searchTerm) {
    return filterBySearchTerm(action, filter.searchTerm);
  }

  return true;
};

const filterActions = (
  actions: Action[],
  filter?: {
    isMainWorkflow: boolean;
    searchTerm?: string;
  },
) => {
  if (actions.length < 1) return [];

  return actions.filter((action) => {
    return combinedFilter(action, filter);
  });
};

const filterActionsByPredicate = (
  obj: Record<string, Action[] | undefined>,
  predicate: (action: Action) => boolean,
  removeEmptyArrays = false,
) =>
  Object.fromEntries(
    Object.entries(obj).reduce(
      (acc, [key, actions]) => {
        const filteredActions = actions?.filter(predicate);
        if (
          !removeEmptyArrays ||
          (filteredActions && filteredActions.length > 0)
        ) {
          acc.push([key, filteredActions]);
        }
        return acc;
      },
      [] as [string, Action[] | undefined][],
    ),
  );

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

export const hasReader = (nodes: Node[] | undefined) => {
  return nodes?.some((node) => node.type === "reader");
};
