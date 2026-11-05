import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type { EventSourceType, Trigger, Variable } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";
import type { PaginationOptions } from "@flow/types/paginationOptions";
import { isDefined } from "@flow/utils";

import type {
  TimeDriverInput,
  UpdateTriggerInput,
  VariableInput,
} from "../__gen__/graphql";
import type { TriggerFragment } from "../__gen__/plugins/graphql-request";
import { toGqlParameterType, toTrigger } from "../convert";
import { useGraphQLContext } from "../provider";

export enum TriggerQueryKeys {
  GetTriggers = "getTriggers",
}

export const TRIGGERS_FETCH_RATE = 15;

// A type the server has no counterpart for is sent as text rather than
// dropped: dropping it lost the variable without any error.
export const toGqlVariableInput = (variables: Variable[]): VariableInput[] =>
  variables.map((variable) => ({
    ...variable,
    type: toGqlParameterType(variable.type) ?? "TEXT",
  }));

const toTriggerPayload = (payload: {
  trigger: TriggerFragment;
  authToken?: string | null;
}) => ({
  trigger: toTrigger(payload.trigger),
  authToken: payload.authToken ?? undefined,
});

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const createTriggerMutation = useMutation({
    mutationFn: async ({
      workspaceId,
      deploymentId,
      description,
      enabled,
      eventSource,
      timeDriverInput,
      variables,
    }: {
      workspaceId: string;
      deploymentId: string;
      description: string;
      enabled: boolean;
      eventSource: EventSourceType;
      timeDriverInput?: TimeDriverInput;
      variables?: Variable[];
    }) => {
      const data = await graphQLContext?.CreateTrigger({
        input: {
          workspaceId,
          deploymentId,
          description,
          enabled,
          eventSource,
          timeDriverInput,
          variables: variables ? toGqlVariableInput(variables) : undefined,
        },
      });

      if (data?.createTrigger) {
        return toTriggerPayload(data.createTrigger);
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
      enabled,
      eventSource,
      timeDriverInput,
      description,
      variables,
    }: {
      triggerId: string;
      enabled?: boolean | null;
      eventSource?: EventSourceType;
      timeDriverInput?: TimeDriverInput;
      description?: string;
      variables?: Variable[];
    }) => {
      const input: UpdateTriggerInput = {
        triggerId,
        enabled,
        eventSource,
        timeDriverInput,
        description,
        variables: variables ? toGqlVariableInput(variables) : undefined,
      };

      const data = await graphQLContext?.UpdateTrigger({
        input,
      });

      if (data?.updateTrigger) {
        return toTriggerPayload(data.updateTrigger);
      }
    },
    onSuccess: (result) => {
      // TODO: Maybe update cache and not refetch? What happens after pagination?
      queryClient.invalidateQueries({
        queryKey: [TriggerQueryKeys.GetTriggers, result?.trigger.workspaceId],
      });
    },
  });

  const regenerateTriggerTokenMutation = useMutation({
    mutationFn: async ({ triggerId }: { triggerId: string }) => {
      const data = await graphQLContext?.RegenerateTriggerToken({
        triggerId,
      });

      if (data?.regenerateTriggerToken) {
        return toTriggerPayload(data.regenerateTriggerToken);
      }
    },
    onSuccess: (result) => {
      queryClient.invalidateQueries({
        queryKey: [TriggerQueryKeys.GetTriggers, result?.trigger.workspaceId],
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

  const useGetTriggersQuery = (
    workspaceId?: string,
    keyword?: string,
    paginationOptions?: PaginationOptions,
  ) =>
    useQuery({
      queryKey: [TriggerQueryKeys.GetTriggers, workspaceId],
      queryFn: async () => {
        const data = await graphQLContext?.GetTriggers({
          workspaceId: workspaceId ?? "",
          keyword,
          pagination: {
            page: paginationOptions?.page ?? 1,
            pageSize: TRIGGERS_FETCH_RATE,
            orderDir: paginationOptions?.orderDir ?? OrderDirection.Desc,
            orderBy: paginationOptions?.orderBy ?? "updatedAt",
          },
        });
        if (!data) return;
        const {
          triggers: {
            nodes,
            pageInfo: { totalCount, totalPages, currentPage },
          },
        } = data;
        const triggers: Trigger[] = nodes
          .filter(isDefined)
          .map((trigger) => toTrigger(trigger));
        return { triggers, totalCount, totalPages, currentPage };
      },
      enabled: !!workspaceId,
    });

  return {
    createTriggerMutation,
    updateTriggerMutation,
    regenerateTriggerTokenMutation,
    deleteTriggerMutation,
    useGetTriggersQuery,
  };
};
