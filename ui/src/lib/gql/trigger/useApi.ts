import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import type {
  CreateTrigger,
  DeleteTrigger,
  TimeInterval,
  Trigger,
  UpdateTrigger,
} from "@flow/types";
import { PaginationOptions } from "@flow/types/paginationOptions";

import { TimeDriverInput } from "../__gen__/graphql";

import { useQueries } from "./useQueries";

export const useTrigger = () => {
  const { toast } = useToast();
  const t = useT();

  const {
    createTriggerMutation,
    updateTriggerMutation,
    deleteTriggerMutation,
    useGetTriggersQuery,
  } = useQueries();

  const createTrigger = async (
    workspaceId: string,
    deploymentId: string,
    description: string,
    timeInterval?: TimeInterval,
    authToken?: string,
  ): Promise<CreateTrigger> => {
    const { mutateAsync, ...rest } = createTriggerMutation;

    try {
      const data = await mutateAsync({
        workspaceId,
        deploymentId,
        timeDriverInput: timeInterval
          ? { interval: timeInterval as TimeDriverInput["interval"] }
          : undefined,
        apiDriverInput: authToken ? { token: authToken } : undefined,
        description,
      });
      toast({
        title: t("Trigger Created"),
        description: t("Trigger has been successfully created."),
      });
      return { trigger: data?.trigger, ...rest };
    } catch (_err) {
      toast({
        title: t("Trigger Could Not Be Created"),
        description: t("There was an error when creating the trigger."),
        variant: "destructive",
      });
      return { trigger: undefined, ...rest };
    }
  };

  const useUpdateTrigger = async (
    triggerId: string,
    timeInterval?: TimeInterval,
    authToken?: string,
    description?: string,
  ): Promise<UpdateTrigger> => {
    const { mutateAsync, ...rest } = updateTriggerMutation;
    try {
      const trigger: Trigger | undefined = await mutateAsync({
        triggerId,
        timeDriverInput: timeInterval
          ? { interval: timeInterval as TimeDriverInput["interval"] }
          : undefined,
        apiDriverInput: authToken ? { token: authToken } : undefined,
        description,
      });
      toast({
        title: t("Trigger Updated"),
        description: t("Trigger has been successfully updated."),
      });
      return { trigger, ...rest };
    } catch (_err) {
      toast({
        title: t("Trigger Could Not Be Updated"),
        description: t("There was an error when updating the trigger."),
        variant: "destructive",
      });
      return { trigger: undefined, ...rest };
    }
  };

  const useDeleteTrigger = async (
    triggerId: string,
    workspaceId: string,
  ): Promise<DeleteTrigger> => {
    const { mutateAsync, ...rest } = deleteTriggerMutation;
    try {
      const data = await mutateAsync({ triggerId, workspaceId });
      toast({
        title: t("Successful Deletion"),
        description: t(
          "Trigger has been successfully deleted from your workspace.",
        ),
      });
      return { success: data.success, ...rest };
    } catch (_err) {
      toast({
        title: t("Trigger Could Not Be Deleted"),
        description: t("There was an error when deleting the trigger."),
        variant: "destructive",
      });
      return { success: false, ...rest };
    }
  };

  const useGetTriggers = (
    workspaceId?: string,
    paginationOptions?: PaginationOptions,
  ) => {
    const { data, ...rest } = useGetTriggersQuery(
      workspaceId,
      paginationOptions,
    );
    return {
      page: data,
      ...rest,
    };
  };

  return {
    createTrigger,
    useGetTriggers,
    useUpdateTrigger,
    useDeleteTrigger,
  };
};
