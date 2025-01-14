import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import type {
  CreateDeployment,
  DeleteDeployment,
  Deployment,
  EngineReadyWorkflow,
  ExecuteDeployment,
  GetDeployments,
  UpdateDeployment,
} from "@flow/types";
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
    useGetDeploymentsInfiniteQuery,
  } = useQueries();

  const createDeployment = async (
    workspaceId: string,
    projectId: string,
    engineReadyWorkflow: EngineReadyWorkflow,
    description?: string,
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
      return { deployment: undefined, ...rest };
    }
  };

  const createDeploymentFromFile = async (
    workspaceId: string,
    workflowFile: File,
    customName?: string,
    description?: string,
  ): Promise<CreateDeployment> => {
    const { mutateAsync, ...rest } = createDeploymentMutation;
    const formData = new FormData();
    formData.append(
      "file",
      new File([workflowFile], customName || workflowFile.name, {
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
      return { deployment: undefined, ...rest };
    }
  };

  const useUpdateDeployment = async (
    deploymentId: string,
    engineReadyWorkflow?: EngineReadyWorkflow,
    description?: string,
  ): Promise<UpdateDeployment> => {
    const { mutateAsync, ...rest } = updateDeploymentMutation;
    try {
      const deployment: Deployment | undefined = await mutateAsync({
        deploymentId,
        engineReadyWorkflow,
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
    createDeploymentFromFile,
    useGetDeploymentsInfinite,
    useUpdateDeployment,
    useDeleteDeployment,
    executeDeployment,
  };
};
