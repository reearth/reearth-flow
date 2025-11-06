import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type { Deployment } from "@flow/types";
import { PaginationOptions } from "@flow/types/paginationOptions";
import { isDefined } from "@flow/utils";

import { ExecuteDeploymentInput } from "../__gen__/graphql";
import {
  DeleteDeploymentInput,
  OrderDirection,
  UpdateDeploymentInput,
} from "../__gen__/plugins/graphql-request";
import { toDeployment, toJob } from "../convert";
import { JobQueryKeys } from "../job/useQueries";
import { ProjectQueryKeys } from "../project/useQueries";
import { useGraphQLContext } from "../provider";

export enum DeploymentQueryKeys {
  GetDeployments = "getDeployments",
  GetDeploymentHead = "getDeploymentHead",
}

export const DEPLOYMENT_FETCH_RATE = 10;

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const createDeploymentMutation = useMutation({
    mutationFn: async ({
      projectId,
      workspaceId,
      file,
      description,
    }: {
      workspaceId: string;
      projectId?: string;
      file: FormData;
      description: string;
    }) => {
      const data = await graphQLContext?.CreateDeployment({
        input: {
          workspaceId,
          projectId,
          file: file.get("file"),
          description,
        },
      });

      if (data?.createDeployment?.deployment) {
        return {
          deployment: toDeployment(data.createDeployment.deployment),
          projectId,
        };
      }
    },
    onSuccess: (result) => {
      // TODO: Maybe update cache and not refetch? What happens after pagination?
      queryClient.invalidateQueries({
        queryKey: [
          DeploymentQueryKeys.GetDeployments,
          result?.deployment?.workspaceId,
        ],
      });
      queryClient.invalidateQueries({
        queryKey: [
          ProjectQueryKeys.GetWorkspaceProjects,
          result?.deployment?.workspaceId,
        ],
      });
      queryClient.invalidateQueries({
        queryKey: [ProjectQueryKeys.GetProject, result?.projectId],
      });
    },
  });

  const updateDeploymentMutation = useMutation({
    mutationFn: async ({
      deploymentId,
      file,
      description,
    }: {
      deploymentId: string;
      file?: FormDataEntryValue;
      description?: string;
    }) => {
      const input: UpdateDeploymentInput = { deploymentId, description, file };

      const data = await graphQLContext?.UpdateDeployment({
        input,
      });

      if (data?.updateDeployment?.deployment) {
        return toDeployment(data.updateDeployment.deployment);
      }
    },
    onSuccess: (deployment) => {
      // TODO: Maybe update cache and not refetch? What happens after pagination?
      queryClient.invalidateQueries({
        queryKey: [DeploymentQueryKeys.GetDeployments, deployment?.workspaceId],
      });
      queryClient.invalidateQueries({
        queryKey: [
          ProjectQueryKeys.GetWorkspaceProjects,
          deployment?.workspaceId,
        ],
      });
      queryClient.invalidateQueries({
        queryKey: [ProjectQueryKeys.GetProject],
      });
    },
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
    onSuccess: ({ workspaceId }) => {
      queryClient.invalidateQueries({
        queryKey: [DeploymentQueryKeys.GetDeployments, workspaceId],
      });
      queryClient.invalidateQueries({
        queryKey: [ProjectQueryKeys.GetWorkspaceProjects, workspaceId],
      });
      queryClient.invalidateQueries({
        queryKey: [ProjectQueryKeys.GetProject],
      });
    },
  });

  const executeDeploymentMutation = useMutation({
    mutationFn: async (input: ExecuteDeploymentInput) => {
      const data = await graphQLContext?.ExecuteDeployment({ input });

      if (data?.executeDeployment?.job) {
        return toJob(data.executeDeployment.job);
      }
    },
    onSuccess: (job) =>
      // TODO: Maybe update cache and not refetch? What happens after pagination?
      queryClient.invalidateQueries({
        queryKey: [JobQueryKeys.GetJobs, job?.workspaceId],
      }),
  });

  const useGetDeploymentsQuery = (
    workspaceId?: string,
    keyword?: string,
    paginationOptions?: PaginationOptions,
  ) =>
    useQuery({
      queryKey: [DeploymentQueryKeys.GetDeployments, workspaceId],
      queryFn: async () => {
        const data = await graphQLContext?.GetDeployments({
          workspaceId: workspaceId ?? "",
          keyword,
          pagination: {
            page: paginationOptions?.page ?? 1,
            pageSize: DEPLOYMENT_FETCH_RATE,
            orderDir: paginationOptions?.orderDir ?? OrderDirection.Desc,
            orderBy: paginationOptions?.orderBy ?? "updatedAt",
          },
        });
        if (!data) return;
        const {
          deployments: {
            nodes,
            pageInfo: { totalCount, currentPage, totalPages },
          },
        } = data;
        const deployments: Deployment[] = nodes
          .filter(isDefined)
          .map((deployment) => toDeployment(deployment));
        return { deployments, totalCount, currentPage, totalPages };
      },
      enabled: !!workspaceId,
    });

  const useGetDeploymentHeadQuery = (
    workspaceId?: string,
    projectId?: string,
  ) =>
    useQuery({
      queryKey: [DeploymentQueryKeys.GetDeploymentHead, workspaceId],
      queryFn: async () => {
        const data = await graphQLContext?.GetDeploymentHead({
          input: {
            workspaceId: workspaceId ?? "",
            projectId,
          },
        });
        if (!data?.deploymentHead) return;
        const deployment: Deployment = toDeployment(data.deploymentHead);
        return { deployment };
      },
      enabled: !!workspaceId,
    });

  return {
    createDeploymentMutation,
    updateDeploymentMutation,
    deleteDeploymentMutation,
    executeDeploymentMutation,
    useGetDeploymentsQuery,
    useGetDeploymentHeadQuery,
  };
};
