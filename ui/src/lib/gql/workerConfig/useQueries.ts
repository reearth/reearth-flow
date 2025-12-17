import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { useGraphQLContext } from "@flow/lib/gql";

import { UpdateWorkerConfigInput } from "../__gen__/graphql";
import { toWorkerConfig } from "../convert";

import { WorkerConfigQueryKeys } from "./useApi";

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const useGetWorkerConfigQuery = () =>
    useQuery({
      queryKey: [WorkerConfigQueryKeys.GetWorkerConfig],
      queryFn: async () => {
        const data = await graphQLContext?.GetWorkerConfig();
        return data?.workerConfig
          ? toWorkerConfig(data.workerConfig)
          : undefined;
      },
    });

  const updateWorkerConfigMutation = useMutation({
    mutationFn: async (input: UpdateWorkerConfigInput) => {
      const data = await graphQLContext?.UpdateWorkerConfig({ input });
      if (!data?.updateWorkerConfig) return;

      return toWorkerConfig(data.updateWorkerConfig.config);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: [WorkerConfigQueryKeys.GetWorkerConfig],
      });
    },
  });

  const deleteWorkerConfigMutation = useMutation({
    mutationFn: async () => {
      const data = await graphQLContext?.DeleteWorkerConfig();
      return {
        id: data?.deleteWorkerConfig?.id,
      };
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: [WorkerConfigQueryKeys.GetWorkerConfig],
        refetchType: "all",
      });
    },
  });

  return {
    useGetWorkerConfigQuery,
    updateWorkerConfigMutation,
    deleteWorkerConfigMutation,
  };
};
