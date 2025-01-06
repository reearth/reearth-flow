import { GetAction, GetActions, GetActionsSegregated } from "@flow/types";

import { useFetch } from "./useFetch";

export const useAction = (lang: string) => {
  const {
    useGetActionsFetch,
    useGetActionsByIdFetch,
    useGetActionsSegregatedFetch,
  } = useFetch();

  const useGetActions = (): GetActions => {
    const { data, ...rest } = useGetActionsFetch(lang);
    return {
      actions: data,
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

  const useGetActionsSegregated = (): GetActionsSegregated => {
    const { data, ...rest } = useGetActionsSegregatedFetch(lang);
    return {
      actions: data,
      ...rest,
    };
  };

  return {
    useGetActions,
    useGetActionById,
    useGetActionsSegregated,
  };
};
