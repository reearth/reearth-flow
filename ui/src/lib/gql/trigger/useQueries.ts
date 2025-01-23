import {
  useInfiniteQuery,
  useMutation,
  useQueryClient,
} from "@tanstack/react-query";

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

const TRIGGERS_FETCH_RATE = 15;

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const createTriggerMutation = useMutation({
    mutationFn: async ({
      workspaceId,
      deploymentId,
      timeDriverInput,
      apiDriverInput,
      description,
    }: {
      workspaceId: string;
      deploymentId: string;
      timeDriverInput?: TimeDriverInput;
      apiDriverInput?: ApiDriverInput;
      description?: string;
    }) => {
      const data = await graphQLContext?.CreateTrigger({
        input: {
          workspaceId,
          deploymentId,
          timeDriverInput,
          apiDriverInput,
          description,
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
      description,
    }: {
      triggerId: string;
      apiDriverInput?: ApiDriverInput;
      timeDriverInput?: TimeDriverInput;
      description?: string;
    }) => {
      const input: UpdateTriggerInput = {
        triggerId,
        apiDriverInput,
        timeDriverInput,
        description,
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

  const useGetTriggersInfiniteQuery = (workspaceId?: string) =>
    useInfiniteQuery({
      queryKey: [TriggerQueryKeys.GetTriggers, workspaceId],
      initialPageParam: null,
      queryFn: async ({ pageParam }) => {
        const data = await graphQLContext?.GetTriggers({
          workspaceId: workspaceId ?? "",
          pagination: {
            first: TRIGGERS_FETCH_RATE,
            after: pageParam,
          },
        });
        if (!data) return;
        const {
          triggers: {
            nodes,
            pageInfo: { endCursor, hasNextPage },
          },
        } = data;
        const triggers: Trigger[] = nodes
          .filter(isDefined)
          .map((trigger) => toTrigger(trigger));
        return { triggers, endCursor, hasNextPage };
      },
      enabled: !!workspaceId,
      getNextPageParam: (lastPage) => {
        if (!lastPage) return undefined;
        const { endCursor, hasNextPage } = lastPage;
        return hasNextPage ? endCursor : undefined;
      },
    });

  return {
    createTriggerMutation,
    updateTriggerMutation,
    deleteTriggerMutation,
    useGetTriggersInfiniteQuery,
  };
};
