import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import type {
  CreateTrigger,
  DeleteTrigger,
  EventSourceType,
  RegenerateTriggerToken,
  TimeInterval,
  UpdateTrigger,
  Variable,
} from "@flow/types";
import type { PaginationOptions } from "@flow/types/paginationOptions";

import type { TimeDriverInput } from "../__gen__/graphql";
import { usePlanLimitMessage } from "../policy/usePlanLimitMessage";
import { toMissingRequiredVariables } from "../workflowVariables/errors";

import { isScheduleNotUpdated } from "./errors";
import { useQueries } from "./useQueries";

export const useTrigger = () => {
  const { toast } = useToast();
  const t = useT();
  const planLimitMessage = usePlanLimitMessage();
  // Names the required variables the server found empty, if that is why it
  // refused.
  const missingRequiredMessage = (err: unknown) => {
    const names = toMissingRequiredVariables(err);
    return names
      ? t("Required variables need a value: {{names}}", {
          names: names.join(", "),
        })
      : undefined;
  };

  const {
    createTriggerMutation,
    updateTriggerMutation,
    regenerateTriggerTokenMutation,
    deleteTriggerMutation,
    useGetTriggersQuery,
  } = useQueries();

  const createTrigger = async (
    workspaceId: string,
    deploymentId: string,
    description: string,
    enabled: boolean,
    eventSource: EventSourceType,
    timeInterval?: TimeInterval,
    variables?: Variable[],
  ): Promise<CreateTrigger> => {
    const { mutateAsync, ...rest } = createTriggerMutation;
    try {
      const data = await mutateAsync({
        workspaceId,
        deploymentId,
        enabled,
        eventSource,
        timeDriverInput: timeInterval
          ? { interval: timeInterval as TimeDriverInput["interval"] }
          : undefined,
        variables,
        description,
      });
      toast({
        title: t("Trigger Created"),
        description: t("Trigger has been successfully created."),
      });
      return { trigger: data?.trigger, authToken: data?.authToken, ...rest };
    } catch (err) {
      const limitMessage = planLimitMessage(err);
      toast({
        title: t("Trigger Could Not Be Created"),
        description:
          limitMessage ??
          missingRequiredMessage(err) ??
          t("There was an error when creating the trigger."),
        variant: limitMessage ? "warning" : "destructive",
      });
      return { trigger: undefined, ...rest };
    }
  };

  const useUpdateTrigger = async (
    triggerId: string,
    enabled?: boolean | null,
    timeInterval?: TimeInterval,
    eventSource?: EventSourceType,
    description?: string,
    variables?: Variable[],
  ): Promise<UpdateTrigger> => {
    const { mutateAsync, ...rest } = updateTriggerMutation;
    try {
      const data = await mutateAsync({
        triggerId,
        enabled,
        eventSource,
        timeDriverInput: timeInterval
          ? { interval: timeInterval as TimeDriverInput["interval"] }
          : undefined,
        description,
        variables,
      });
      toast({
        title: t("Trigger Updated"),
        description: t("Trigger has been successfully updated."),
      });
      return { trigger: data?.trigger, authToken: data?.authToken, ...rest };
    } catch (err) {
      if (isScheduleNotUpdated(err)) {
        toast({
          title: t("Trigger Saved, Schedule Not Updated"),
          description: t(
            "The trigger was saved, but its schedule could not be updated. Save it again to retry.",
          ),
          variant: "warning",
        });
        return { trigger: undefined, ...rest };
      }
      const limitMessage = planLimitMessage(err);
      toast({
        title: t("Trigger Could Not Be Updated"),
        description:
          limitMessage ??
          missingRequiredMessage(err) ??
          t("There was an error when updating the trigger."),
        variant: limitMessage ? "warning" : "destructive",
      });
      return { trigger: undefined, ...rest };
    }
  };

  const regenerateTriggerToken = async (
    triggerId: string,
  ): Promise<RegenerateTriggerToken> => {
    const { mutateAsync, ...rest } = regenerateTriggerTokenMutation;
    try {
      const data = await mutateAsync({ triggerId });
      toast({
        title: t("Auth Token has been re-generated!"),
      });
      return { trigger: data?.trigger, authToken: data?.authToken, ...rest };
    } catch (_err) {
      toast({
        title: t("The attempt to re-generate the auth token has failed."),
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
    keyword?: string,
    paginationOptions?: PaginationOptions,
  ) => {
    const { data, ...rest } = useGetTriggersQuery(
      workspaceId,
      keyword,
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
    regenerateTriggerToken,
    useDeleteTrigger,
  };
};
