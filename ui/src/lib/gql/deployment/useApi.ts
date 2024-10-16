import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import type {
  CreateDeployment,
  ExecuteDeployment,
  GetDeployments,
} from "@flow/types";

import { ExecuteDeploymentInput } from "../__gen__/graphql";

import { useQueries } from "./useQueries";

export const useDeployment = () => {
  const { toast } = useToast();
  const t = useT();

  const {
    createDeploymentMutation,
    executeDeploymentMutation,
    useGetDeploymentsInfiniteQuery,
  } = useQueries();

  const createDeployment = async (
    workspaceId: string,
    projectId: string,
    workflow: FormData,
  ): Promise<CreateDeployment> => {
    const { mutateAsync, ...rest } = createDeploymentMutation;

    try {
      const deployment = await mutateAsync({
        projectId,
        workspaceId,
        metaFile: undefined, // TODO: Add meta file
        workflowsZip: workflow,
      });
      toast({
        title: t("Deployment Created"),
        description: t("Deployment has been successfully created."),
      });
      return { deployment, ...rest };
    } catch (_err) {
      return { deployment: undefined, ...rest };
    }
  };

  const useGetDeploymentsInfinite = (projectId?: string): GetDeployments => {
    const { data, ...rest } = useGetDeploymentsInfiniteQuery(projectId);
    return {
      pages: data?.pages,
      ...rest,
    };
  };

  const executeDeployment = async (
    input: ExecuteDeploymentInput,
  ): Promise<ExecuteDeployment> => {
    const { mutateAsync, ...rest } = executeDeploymentMutation;
    try {
      const job = await mutateAsync(input);
      toast({
        title: t("Deployment Executed"),
        description: t("Deployment has been successfully executed."),
      });
      return { job, ...rest };
    } catch (_err) {
      return { job: undefined, ...rest };
    }
  };

  return {
    createDeployment,
    useGetDeploymentsInfinite,
    executeDeployment,
  };
};
