import {
  useInfiniteQuery,
  useMutation,
  useQueryClient,
} from "@tanstack/react-query";

import type { Deployment } from "@flow/types";
import { isDefined } from "@flow/utils";

import { ExecuteDeploymentInput } from "../__gen__/graphql";
import {
  DeleteDeploymentInput,
  UpdateDeploymentInput,
} from "../__gen__/plugins/graphql-request";
import { toDeployment, toJob } from "../convert";
import { JobQueryKeys } from "../job/useQueries";
import { ProjectQueryKeys } from "../project/useQueries";
import { useGraphQLContext } from "../provider";

export enum DeploymentQueryKeys {
  GetDeployments = "getDeployments",
}

const DEPLOYMENT_FETCH_RATE = 10;

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
      description?: string;
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

  const useGetDeploymentsInfiniteQuery = (workspaceId?: string) =>
    useInfiniteQuery({
      queryKey: [DeploymentQueryKeys.GetDeployments, workspaceId],
      initialPageParam: 1,
      queryFn: async ({ pageParam }) => {
        const data = await graphQLContext?.GetDeployments({
          workspaceId: workspaceId ?? "",
          pagination: {
            page: pageParam,
            pageSize: DEPLOYMENT_FETCH_RATE,
            // orderDir: "ASC",
          },
        });
        if (!data) return;
        const {
          deploymentsPage: {
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
      getNextPageParam: (lastPage) => {
        if (!lastPage) return undefined;
        if ((lastPage.currentPage ?? 0) < (lastPage.totalPages ?? 0)) {
          return (lastPage.currentPage ?? 0) + 1;
        }
        return undefined;
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
