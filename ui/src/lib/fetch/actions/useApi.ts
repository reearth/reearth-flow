import { GetAction, GetActions, GetActionSegregated } from "@flow/types";

import { useFetch } from "./useFetch";

export const useAction = () => {
  const {
    useGetActionsFetch,
    useGetActionsByIdFetch,
    useGetActionsSegregatedFetch,
  } = useFetch();

  const useGetActions = (): GetActions => {
    const { data, ...rest } = useGetActionsFetch();
    return {
      actions: data,
      ...rest,
    };
  };

  const useGetActionById = (id: string): GetAction => {
    const { data, ...rest } = useGetActionsByIdFetch(id);
    return {
      action: data,
      ...rest,
    };
  };

  const useGetActionSegregated = (): GetActionSegregated => {
    const { data, ...rest } = useGetActionsSegregatedFetch();
    return {
      actions: data,
      ...rest,
    };
  };

  return {
    useGetActions,
    useGetActionById,
    useGetActionSegregated,
  };
};
