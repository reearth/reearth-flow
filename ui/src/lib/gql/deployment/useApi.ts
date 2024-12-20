import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import type {
  CreateDeployment,
  DeleteDeployment,
  Deployment,
  ExecuteDeployment,
  GetDeployments,
  UpdateDeployment,
} from "@flow/types";
import { yamlToFormData } from "@flow/utils/yamlToFormData";

import { ExecuteDeploymentInput } from "../__gen__/graphql";

import { useQueries } from "./useQueries";

export const useDeployment = () => {
  const { toast } = useToast();
  const t = useT();

  const {
    createDeploymentMutation,
    updateDeploymentMutation,
    deleteDeploymentMutation,
    executeDeploymentMutation,
    useGetDeploymentsInfiniteQuery,
    useGetJobsInfiniteQuery,
  } = useQueries();

  const createDeployment = async (
    workspaceId: string,
    projectId: string,
    workflowId: string,
    workflow: string,
    description?: string,
  ): Promise<CreateDeployment> => {
    const { mutateAsync, ...rest } = createDeploymentMutation;

    try {
      const formData = yamlToFormData(workflow, workflowId);

      const data = await mutateAsync({
        workspaceId,
        projectId,
        file: formData,
        description,
      });
      toast({
        title: t("Deployment Created"),
        description: t("Deployment has been successfully created."),
      });
      return { deployment: data?.deployment, ...rest };
    } catch (_err) {
      return { deployment: undefined, ...rest };
    }
  };

  const useUpdateDeployment = async (
    deploymentId: string,
    workflowId?: string,
    workflowYaml?: string,
    description?: string,
  ): Promise<UpdateDeployment> => {
    const { mutateAsync, ...rest } = updateDeploymentMutation;
    try {
      const deployment: Deployment | undefined = await mutateAsync({
        deploymentId,
        workflowId,
        workflowYaml,
        description,
      });
      toast({
        title: t("Deployment Updated"),
        description: t("Deployment has been successfully updated."),
      });
      return { deployment, ...rest };
    } catch (_err) {
      return { deployment: undefined, ...rest };
    }
  };

  const useDeleteDeployment = async (
    deploymentId: string,
    workspaceId: string,
  ): Promise<DeleteDeployment> => {
    const { mutateAsync, ...rest } = deleteDeploymentMutation;
    try {
      const data = await mutateAsync({ deploymentId, workspaceId });
      toast({
        title: t("Successful Deletion"),
        description: t(
          "Deployment has been successfully deleted from your workspace.",
        ),
        variant: "destructive",
      });
      return { deploymentId: data.deploymentId, ...rest };
    } catch (_err) {
      return { deploymentId: undefined, ...rest };
    }
  };

  const useGetDeploymentsInfinite = (workspaceId?: string): GetDeployments => {
    const { data, ...rest } = useGetDeploymentsInfiniteQuery(workspaceId);
    return {
      pages: data?.pages,
      ...rest,
    };
  };

  const useGetJobsInfinite = (workspaceId?: string) => {
    const { data, ...rest } = useGetJobsInfiniteQuery(workspaceId);
    return {
      pages: data?.pages,
      ...rest,
    };
  };

  const executeDeployment = async (
    input: ExecuteDeploymentInput,
  ): Promise<ExecuteDeployment> => {
    const { mutateAsync, ...rest } = executeDeploymentMutation;
    console.log("DEPLOYMENT EXECUTE", input);
    try {
      const job = await mutateAsync(input);
      console.log("job", job);
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
    useGetJobsInfinite,
    useUpdateDeployment,
    useDeleteDeployment,
    executeDeployment,
  };
};
