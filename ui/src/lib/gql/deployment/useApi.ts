import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import type {
  CreateDeployment,
  DeleteDeployment,
  Deployment,
  EngineReadyWorkflow,
  ExecuteDeployment,
  UpdateDeployment,
} from "@flow/types";
import { PaginationOptions } from "@flow/types/paginationOptions";
import { jsonToFormData } from "@flow/utils/jsonToFormData";

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
    useGetDeploymentsQuery,
  } = useQueries();

  const createDeployment = async (
    workspaceId: string,
    projectId: string,
    engineReadyWorkflow: EngineReadyWorkflow,
    description: string,
  ): Promise<CreateDeployment> => {
    const { mutateAsync, ...rest } = createDeploymentMutation;

    try {
      const formData = jsonToFormData(
        engineReadyWorkflow,
        engineReadyWorkflow.id,
      );

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
      toast({
        title: t("Deployment Could Not Be Created"),
        description: t("There was an error when creating the deployment."),
        variant: "destructive",
      });
      return { deployment: undefined, ...rest };
    }
  };

  const createDeploymentFromFile = async (
    workspaceId: string,
    workflowFile: File,
    description: string,
  ): Promise<CreateDeployment> => {
    const { mutateAsync, ...rest } = createDeploymentMutation;
    const formData = new FormData();
    formData.append(
      "file",
      new File([workflowFile], workflowFile.name, {
        type: workflowFile.type,
      }),
    );

    try {
      const data = await mutateAsync({
        workspaceId,
        file: formData,
        description,
      });
      toast({
        title: t("Deployment Created"),
        description: t("Deployment has been successfully created."),
      });
      return { deployment: data?.deployment, ...rest };
    } catch (_err) {
      toast({
        title: t("Deployment Could Not Be Created"),
        description: t("There was an error when creating the deployment."),
        variant: "destructive",
      });
      return { deployment: undefined, ...rest };
    }
  };

  const useUpdateDeployment = async (
    deploymentId: string,
    file?: FormDataEntryValue,
    description?: string,
  ): Promise<UpdateDeployment> => {
    const { mutateAsync, ...rest } = updateDeploymentMutation;
    try {
      const deployment: Deployment | undefined = await mutateAsync({
        deploymentId,
        file,
        description,
      });
      toast({
        title: t("Deployment Updated"),
        description: t("Deployment has been successfully updated."),
      });
      return { deployment, ...rest };
    } catch (_err) {
      toast({
        title: t("Deployment Could Not Be Updated"),
        description: t("There was an error when updating the deployment."),
        variant: "destructive",
      });
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
      });
      return { deploymentId: data.deploymentId, ...rest };
    } catch (_err) {
      toast({
        title: t("Deployment Could Not Be Deleted"),
        description: t("There was an error when deleting the deployment."),
        variant: "destructive",
      });
      return { deploymentId: undefined, ...rest };
    }
  };

  const useGetDeployments = (
    workspaceId?: string,
    paginationOptions?: PaginationOptions,
  ) => {
    const { data, ...rest } = useGetDeploymentsQuery(
      workspaceId,
      paginationOptions,
    );
    return {
      page: data,
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
      toast({
        title: t("Deployment Could Not Be Executed"),
        description: t(
          "There was an error when attempting to run the current deployment.",
        ),
        variant: "destructive",
      });
      return { job: undefined, ...rest };
    }
  };

  return {
    createDeployment,
    createDeploymentFromFile,
    useGetDeployments,
    useUpdateDeployment,
    useDeleteDeployment,
    executeDeployment,
  };
};
