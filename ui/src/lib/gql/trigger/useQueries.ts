import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type { Trigger } from "@flow/types";
import { isDefined } from "@flow/utils";

import type {
  ApiDriverInput,
  TimeDriverInput,
  UpdateTriggerInput,
} from "../__gen__/graphql";
import { toTrigger } from "../convert";
import { useGraphQLContext } from "../provider";

export enum TriggerQueryKeys {
  GetTriggers = "getTriggers",
}

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const createTriggerMutation = useMutation({
    mutationFn: async ({
      workspaceId,
      deploymentId,
      timeDriverInput,
      apiDriverInput,
    }: {
      workspaceId: string;
      deploymentId: string;
      timeDriverInput?: TimeDriverInput;
      apiDriverInput?: ApiDriverInput;
    }) => {
      const data = await graphQLContext?.CreateTrigger({
        input: {
          workspaceId,
          deploymentId,
          timeDriverInput,
          apiDriverInput,
        },
      });

      if (data?.createTrigger) {
        return {
          trigger: toTrigger(data.createTrigger),
        };
      }
    },
    onSuccess: (result) => {
      queryClient.invalidateQueries({
        queryKey: [TriggerQueryKeys.GetTriggers, result?.trigger.workspaceId],
      });
    },
  });

  const updateTriggerMutation = useMutation({
    mutationFn: async ({
      triggerId,
      apiDriverInput,
      timeDriverInput,
    }: {
      triggerId: string;
      apiDriverInput?: ApiDriverInput;
      timeDriverInput?: TimeDriverInput;
    }) => {
      const input: UpdateTriggerInput = {
        triggerId,
        apiDriverInput,
        timeDriverInput,
      };

      const data = await graphQLContext?.UpdateTrigger({
        input,
      });

      if (data?.updateTrigger) {
        return toTrigger(data.updateTrigger);
      }
    },
    onSuccess: (trigger) => {
      // TODO: Maybe update cache and not refetch? What happens after pagination?
      queryClient.invalidateQueries({
        queryKey: [TriggerQueryKeys.GetTriggers, trigger?.workspaceId],
      });
    },
  });

  const deleteTriggerMutation = useMutation({
    mutationFn: async ({
      triggerId,
      workspaceId,
    }: { triggerId: string } & { workspaceId: string }) => {
      const data = await graphQLContext?.DeleteTrigger({
        triggerId,
      });
      return {
        success: data?.deleteTrigger,
        workspaceId,
      };
    },
    onSuccess: ({ workspaceId }) => {
      queryClient.invalidateQueries({
        queryKey: [TriggerQueryKeys.GetTriggers, workspaceId],
      });
    },
  });

  const useGetTriggersQuery = (workspaceId?: string) =>
    useQuery({
      queryKey: [TriggerQueryKeys.GetTriggers, workspaceId],
      queryFn: async () => {
        const data = await graphQLContext?.GetTriggers({
          workspaceId: workspaceId ?? "",
        });
        if (!data) return;

        const triggers: Trigger[] = data.triggers
          .filter(isDefined)
          .map((trigger) => toTrigger(trigger));
        return { triggers };
      },
      enabled: !!workspaceId,
    });

  return {
    createTriggerMutation,
    updateTriggerMutation,
    deleteTriggerMutation,
    useGetTriggersQuery,
  };
};
