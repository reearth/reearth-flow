import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type { UserParameter, VarType } from "@flow/types";
import { isDefined } from "@flow/utils";

import { toGqlParameterType, toUserParameter } from "../convert";
import { useGraphQLContext } from "../provider";

export enum ParameterQueryKeys {
  GetParameters = "getParameters",
}

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const useUserParametersQuery = (projectId?: string) =>
    useQuery({
      queryKey: [ParameterQueryKeys.GetParameters, projectId],
      queryFn: async () => {
        const data = await graphQLContext?.GetProjectParameters({
          projectId: projectId ?? "",
        });

        if (!data) return;
        const { parameters } = data;
        const userParameters: UserParameter[] = parameters
          .filter(isDefined)
          .map((p) => toUserParameter(p));

        return userParameters;
      },
      enabled: !!projectId,
      refetchOnMount: false,
      refetchOnWindowFocus: false,
    });

  const createUserParametersMutation = useMutation({
    mutationFn: async ({
      projectId,
      name,
      value,
      type,
      required,
      index,
    }: {
      projectId: string;
      name: string;
      value: any;
      type: VarType;
      required: boolean;
      index: number;
    }) => {
      const gqlType = toGqlParameterType(type);
      if (!gqlType) return;
      const data = await graphQLContext?.CreateUserParameters({
        projectId,
        input: {
          name,
          value,
          type: gqlType,
          required,
          index,
        },
      });

      if (data?.declareParameter) {
        return toUserParameter(data?.declareParameter);
      }
    },
    onSuccess: (parameterDocument) => {
      if (parameterDocument) {
        queryClient.invalidateQueries({
          queryKey: [
            ParameterQueryKeys.GetParameters,
            parameterDocument.projectId,
          ],
        });
      }
    },
  });

  return {
    useUserParametersQuery,
    createUserParametersMutation,
  };
};
