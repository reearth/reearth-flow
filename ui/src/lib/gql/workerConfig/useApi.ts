import { useToast } from "@flow/features/NotificationSystem/useToast";
import {
  DeleteWorkerConfig,
  GetWorkerConfig,
  WorkerConfigMutation,
} from "@flow/types";

import { UpdateWorkerConfigInput } from "../__gen__/graphql";

import { useQueries } from "./useQueries";

export enum WorkerConfigQueryKeys {
  GetWorkerConfig = "getWorkerConfig",
}

export const useWorkerConfig = () => {
  const { toast } = useToast();

  const {
    useGetWorkerConfigQuery,
    updateWorkerConfigMutation,
    deleteWorkerConfigMutation,
  } = useQueries();

  const useGetWorkerConfig = (): GetWorkerConfig => {
    const {
      data: workerConfig,
      isLoading,
      ...rest
    } = useGetWorkerConfigQuery();
    return {
      workerConfig,
      isLoading,
      ...rest,
    };
  };

  const updateWorkerConfig = async (
    input: UpdateWorkerConfigInput,
  ): Promise<WorkerConfigMutation> => {
    const { mutateAsync, ...rest } = updateWorkerConfigMutation;
    try {
      const data = await mutateAsync(input);
      toast({
        title: "Configuration Updated",
        description: "Configuration has been successfully Updated.",
      });
      return { config: data, ...rest };
    } catch (_err) {
      toast({
        title: "Configuration Could Not Be Updated",
        description: "There was an error when updating the configuration.",
        variant: "destructive",
      });
      return { config: undefined, ...rest };
    }
  };

  const deleteWorkerConfig = async (): Promise<DeleteWorkerConfig> => {
    const { mutateAsync, ...rest } = deleteWorkerConfigMutation;
    try {
      const data = await mutateAsync();
      toast({
        title: "Configuration Deleted",
        description: "Configuration has been successfully deleted.",
      });
      return { id: data.id ?? "", ...rest };
    } catch (_err) {
      toast({
        title: "Configuration Could Not Be Deleted",
        description: "There was an error when deleting the configuration.",
        variant: "destructive",
      });
      return { id: "", ...rest };
    }
  };

  return {
    useGetWorkerConfig,
    updateWorkerConfig,
    deleteWorkerConfig,
  };
};
