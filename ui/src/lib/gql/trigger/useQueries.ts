import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type { Trigger } from "@flow/types";
import {
  OrderDirection,
  type PaginationOptions,
} from "@flow/types/paginationOptions";
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

export const TRIGGERS_FETCH_RATE = 15;

// Helper function to convert all variable values to strings for the backend
const convertVariablesToStrings = (
  variables?: Record<string, any>,
): Record<string, string> | undefined => {
  if (!variables) return undefined;
  const stringifiedVars: Record<string, string> = {};
  Object.entries(variables).forEach(([key, value]) => {
    if (typeof value === "string") {
      stringifiedVars[key] = value;
    } else if (typeof value === "object") {
      stringifiedVars[key] = JSON.stringify(value);
    } else if (value === null) {
      stringifiedVars[key] = "null";
    } else {
      stringifiedVars[key] = String(value);
    }
  });

  return stringifiedVars;
};

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
      variables,
    }: {
      workspaceId: string;
      deploymentId: string;
      timeDriverInput?: TimeDriverInput;
      apiDriverInput?: ApiDriverInput;
      description: string;
      variables?: Record<string, any>;
    }) => {
      const data = await graphQLContext?.CreateTrigger({
        input: {
          workspaceId,
          deploymentId,
          timeDriverInput,
          apiDriverInput,
          description,
          variables: convertVariablesToStrings(variables),
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
      variables,
    }: {
      triggerId: string;
      apiDriverInput?: ApiDriverInput;
      timeDriverInput?: TimeDriverInput;
      description?: string;
      variables?: Record<string, any>;
    }) => {
      const input: UpdateTriggerInput = {
        triggerId,
        apiDriverInput,
        timeDriverInput,
        description,
        variables: convertVariablesToStrings(variables),
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
    deleteTriggerMutation,
    useGetTriggersQuery,
  };
};
