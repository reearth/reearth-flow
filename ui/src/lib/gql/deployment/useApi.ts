import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import type {
  CreateDeployment,
  DeleteDeployment,
  Deployment,
  EngineReadyWorkflow,
  ExecuteDeployment,
  UpdateDeployment,
  Variable,
} from "@flow/types";
import type { PaginationOptions } from "@flow/types/paginationOptions";
import { jsonToFormData } from "@flow/utils/jsonToFormData";

import { usePlanLimitMessage } from "../policy/usePlanLimitMessage";
import {
  toInvalidVariableValues,
  toMissingRequiredVariables,
  toUnknownVariables,
} from "../workflowVariables/errors";

import { isDeploymentHasTriggersError } from "./errors";
import { useQueries } from "./useQueries";

export const useDeployment = () => {
  const { toast } = useToast();
  const t = useT();
  const planLimitMessage = usePlanLimitMessage();
  // Names the required variables the server found empty, if that is why it
  // refused.
  const missingRequiredMessage = (err: unknown) => {
    const names = toMissingRequiredVariables(err);
    return names
      ? t("Required variables need a value: {{names}}", {
          names: names.join(", "),
        })
      : undefined;
  };
  // Names the variables a run was refused for: ones the deployment does not
  // define, or values that do not fit their type.
  const runVariablesMessage = (err: unknown) => {
    const unknown = toUnknownVariables(err);
    const invalid = toInvalidVariableValues(err);
    const messages = [
      unknown?.length &&
        t("This deployment has no variables named: {{names}}", {
          names: unknown.join(", "),
        }),
      invalid?.length &&
        t("These variables were given a value of the wrong type: {{names}}", {
          names: invalid.join(", "),
        }),
    ].filter(Boolean);
    return messages.length ? messages.join(" ") : undefined;
  };

  const {
    createDeploymentMutation,
    updateDeploymentMutation,
    deleteDeploymentMutation,
    executeDeploymentMutation,
    useGetDeploymentsQuery,
    useGetDeploymentHeadQuery,
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
    } catch (err) {
      toast({
        title: t("Deployment Could Not Be Deleted"),
        description: isDeploymentHasTriggersError(err)
          ? t(
              "Triggers still run this deployment. Delete those triggers first, then try again.",
            )
          : t("There was an error when deleting the deployment."),
        variant: "destructive",
      });
      return { deploymentId: undefined, ...rest };
    }
  };

  const useGetDeployments = (
    workspaceId?: string,
    keyword?: string,
    paginationOptions?: PaginationOptions,
  ) => {
    const { data, ...rest } = useGetDeploymentsQuery(
      workspaceId,
      keyword,
      paginationOptions,
    );
    return {
      page: data,
      ...rest,
    };
  };

  const useGetDeploymentHead = (workspaceId?: string, projectId?: string) => {
    const { data, ...rest } = useGetDeploymentHeadQuery(workspaceId, projectId);
    return {
      deployment: data?.deployment,
      ...rest,
    };
  };

  const executeDeployment = async (input: {
    deploymentId: string;
    variables?: Variable[];
  }): Promise<ExecuteDeployment> => {
    const { mutateAsync, ...rest } = executeDeploymentMutation;
    try {
      const job = await mutateAsync(input);
      toast({
        title: t("Deployment Executed"),
        description: t("Deployment has been successfully executed."),
      });
      return { job, ...rest };
    } catch (err) {
      toast({
        title: t("Deployment Could Not Be Executed"),
        description:
          planLimitMessage(err) ??
          missingRequiredMessage(err) ??
          runVariablesMessage(err) ??
          t(
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
    useGetDeploymentHead,
    useUpdateDeployment,
    useDeleteDeployment,
    executeDeployment,
  };
};
