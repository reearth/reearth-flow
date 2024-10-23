import {
  useInfiniteQuery,
  useMutation,
  useQueryClient,
} from "@tanstack/react-query";
import { useCallback } from "react";

import { Deployment } from "@flow/types";
import { isDefined } from "@flow/utils";
import { yamlToFormData } from "@flow/utils/yamlToFormData";

import { DeploymentFragment, ExecuteDeploymentInput } from "../__gen__/graphql";
import { DeleteDeploymentInput } from "../__gen__/plugins/graphql-request";
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
      projectName: deployment.project?.name,
      workflowUrl: deployment.workflowUrl,
      version: deployment.version,
      createdAt: deployment.createdAt,
      updatedAt: deployment.updatedAt,
    }),
    [],
  );

  const createDeploymentMutation = useMutation({
    mutationFn: async ({
      projectId,
      workspaceId,
      file,
    }: {
      workspaceId: string;
      projectId: string;
      file: FormData;
    }) => {
      const data = await graphQLContext?.CreateDeployment({
        input: {
          workspaceId,
          projectId,
          file: file.get("file"),
        },
      });

      console.log("Data", data);
      if (data?.createDeployment?.deployment) {
        return createNewDeploymentObject(data.createDeployment.deployment);
      }
    },
    onSuccess: (deployment) => {
      // TODO: Maybe update cache and not refetch? What happens after pagination?
      queryClient.invalidateQueries({
        queryKey: [DeploymentQueryKeys.GetDeployments, deployment?.workspaceId],
      });
    },
  });

  const updateDeploymentMutation = useMutation({
    mutationFn: async ({
      deploymentId,
      workflowYaml,
      workflowId,
    }: {
      deploymentId: string;
      workflowId: string;
      workflowYaml: string;
    }) => {
      const formData = yamlToFormData(workflowYaml, workflowId);

      const data = await graphQLContext?.UpdateDeployment({
        input: { deploymentId, file: formData.get("file") },
      });

      if (data?.updateDeployment?.deployment) {
        return data?.updateDeployment?.deployment;
      }
    },
    onSuccess: (deployment) =>
      // TODO: Maybe update cache and not refetch? What happens after pagination?
      queryClient.invalidateQueries({
        queryKey: [DeploymentQueryKeys.GetDeployments, deployment?.workspaceId],
      }),
  });

  const deleteDeploymentMutation = useMutation({
    mutationFn: async ({
      deploymentId,
      workspaceId,
    }: DeleteDeploymentInput & { workspaceId: string }) => {
      const data = await graphQLContext?.DeleteDeployment({
        input: { deploymentId },
      });
      return {
        deploymentId: data?.deleteDeployment?.deploymentId,
        workspaceId,
      };
    },
    onSuccess: ({ workspaceId }) =>
      queryClient.invalidateQueries({
        queryKey: [DeploymentQueryKeys.GetDeployments, workspaceId],
      }),
  });

  const executeDeploymentMutation = useMutation({
    mutationFn: async (input: ExecuteDeploymentInput) => {
      const data = await graphQLContext?.ExecuteDeployment({ input });

      if (data?.executeDeployment?.job) {
        return createNewJobObject(data.executeDeployment.job);
      }
    },
    onSuccess: (job) =>
      // TODO: Maybe update cache and not refetch? What happens after pagination?
      queryClient.invalidateQueries({
        queryKey: ["getJobs", job?.workspaceId],
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
    updateDeploymentMutation,
    deleteDeploymentMutation,
    executeDeploymentMutation,
    useGetDeploymentsInfiniteQuery,
  };
};
