import {
  useInfiniteQuery,
  useMutation,
  useQueryClient,
} from "@tanstack/react-query";
import { useCallback } from "react";

import { Deployment } from "@flow/types";
import { isDefined } from "@flow/utils";

import {
  CreateDeploymentInput,
  DeploymentFragment,
  ExecuteDeploymentInput,
} from "../__gen__/graphql";
import { createNewJobObject } from "../job/useQueries";
import { useGraphQLContext } from "../provider";

enum DeploymentQueryKeys {
  GetDeployments = "getDeployments",
}

const DEPLOYMENT_FETCH_RATE = 10;

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const createNewDeploymentObject = useCallback(
    (deployment: DeploymentFragment): Deployment => ({
      id: deployment.id,
      workspaceId: deployment.workspaceId,
      projectId: deployment.projectId,
      workflowId: deployment.workflowId,
      version: deployment.version,
      createdAt: deployment.createdAt,
      updatedAt: deployment.updatedAt,
    }),
    [],
  );

  const createDeploymentMutation = useMutation({
    mutationFn: async (input: CreateDeploymentInput) => {
      const data = await graphQLContext?.CreateDeployment({ input });

      if (data?.createDeployment?.deployment) {
        return createNewDeploymentObject(data.createDeployment.deployment);
      }
    },
    onSuccess: (deployment) => {
      // TODO: Maybe update cache and not refetch? What happens after pagination?
      queryClient.invalidateQueries({
        queryKey: ["getDeployments", deployment?.projectId],
      });
    },
  });

  const executeDeploymentMutation = useMutation({
    mutationFn: async (input: ExecuteDeploymentInput) => {
      const data = await graphQLContext?.ExecuteDeployment({ input });

      if (data?.executeDeployment?.job) {
        return createNewJobObject(data.executeDeployment.job);
      }
    },
    onSuccess: (deployment) =>
      // TODO: Maybe update cache and not refetch? What happens after pagination?
      queryClient.invalidateQueries({
        queryKey: ["getJobs", deployment?.workspaceId],
      }),
  });

  const useGetDeploymentsInfiniteQuery = (workspaceId?: string) =>
    useInfiniteQuery({
      queryKey: [DeploymentQueryKeys.GetDeployments, workspaceId],
      initialPageParam: null,
      queryFn: async ({ pageParam }) => {
        const data = await graphQLContext?.GetDeployments({
          workspaceId: workspaceId ?? "",
          pagination: {
            first: DEPLOYMENT_FETCH_RATE,
            after: pageParam,
          },
        });
        if (!data) return;
        const {
          deployments: {
            nodes,
            pageInfo: { endCursor, hasNextPage },
          },
        } = data;
        const deployments: Deployment[] = nodes
          .filter(isDefined)
          .map((deployment) => createNewDeploymentObject(deployment));
        return { deployments, endCursor, hasNextPage };
      },
      enabled: !!workspaceId,
      getNextPageParam: (lastPage) => {
        if (!lastPage) return undefined;
        const { endCursor, hasNextPage } = lastPage;
        return hasNextPage ? endCursor : undefined;
      },
    });

  return {
    createDeploymentMutation,
    executeDeploymentMutation,
    useGetDeploymentsInfiniteQuery,
  };
};
