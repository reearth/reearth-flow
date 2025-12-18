import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type { AnyWorkflowVariable, VarType } from "@flow/types";
import { isDefined } from "@flow/utils";

import { toGqlParameterType, toWorkflowVariable } from "../convert";
import { useGraphQLContext } from "../provider";

export enum ParameterQueryKeys {
  GetParameters = "getParameters",
}

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const useWorkflowVariablesQuery = (projectId?: string) =>
    useQuery({
      queryKey: [ParameterQueryKeys.GetParameters, projectId],
      queryFn: async () => {
        const data = await graphQLContext?.GetWorkflowParameters({
          projectId: projectId ?? "",
        });

        if (!data) return;
        const { parameters } = data;
        const workflowVars: AnyWorkflowVariable[] = parameters
          .filter(isDefined)
          .map((p) => toWorkflowVariable(p));

        return workflowVars;
      },
      enabled: !!projectId,
      refetchOnMount: false,
      refetchOnWindowFocus: false,
    });

  const createWorkflowVariablesMutation = useMutation({
    mutationFn: async ({
      projectId,
      name,
      defaultValue,
      type,
      required,
      publicValue,
      index,
      config,
    }: {
      projectId: string;
      name: string;
      defaultValue: any;
      type: VarType;
      required: boolean;
      publicValue: boolean;
      index: number;
      config?: any;
    }) => {
      const gqlType = toGqlParameterType(type);
      if (!gqlType) return;
      const data = await graphQLContext?.CreateWorkflowVariable({
        projectId,
        input: {
          name,
          defaultValue,
          type: gqlType,
          required,
          public: publicValue,
          index,
          config,
        },
      });

      if (data?.declareParameter) {
        return toWorkflowVariable(data?.declareParameter);
      }
    },
    onSuccess: (parameterDocument) => {
      if (parameterDocument) {
        queryClient.invalidateQueries({
          queryKey: [
            ParameterQueryKeys.GetParameters,
            parameterDocument.projectId,
          ],
        });
      }
    },
  });

  const deleteWorkflowVariableMutation = useMutation({
    mutationFn: async ({
      paramId,
      projectId,
    }: {
      paramId: string;
      projectId: string;
    }) => {
      const data = await graphQLContext?.DeleteWorkflowVariable({
        input: {
          paramId,
        },
      });
      if (data?.removeParameter === true) {
        return { success: true, projectId };
      }
      throw new Error("Failed to delete workflow variable");
    },
    onSuccess: (result) => {
      if (result?.success && result?.projectId) {
        queryClient.invalidateQueries({
          queryKey: [ParameterQueryKeys.GetParameters, result.projectId],
        });
        queryClient.invalidateQueries({
          queryKey: [ParameterQueryKeys.GetParameters],
        });
      }
    },
  });

  const updateMultipleWorkflowVariablesMutation = useMutation({
    mutationFn: async (input: {
      projectId: string;
      creates?: {
        name: string;
        defaultValue: any;
        type: VarType;
        required: boolean;
        publicValue: boolean;
        index?: number;
        config?: any;
      }[];
      updates?: {
        paramId: string;
        name?: string;
        defaultValue?: any;
        type?: VarType;
        required?: boolean;
        publicValue?: boolean;
        config?: any;
      }[];
      deletes?: string[];
      reorders?: {
        paramId: string;
        newIndex: number;
      }[];
    }) => {
      const multiInput: any = {
        projectId: input.projectId,
      };

      if (input.creates && input.creates.length > 0) {
        multiInput.creates = input.creates.map((create) => {
          const gqlType = toGqlParameterType(create.type);
          if (!gqlType)
            throw new Error(`Invalid parameter type: ${create.type}`);
          return {
            name: create.name,
            defaultValue: create.defaultValue,
            type: gqlType,
            required: create.required,
            public: create.publicValue,
            index: create.index,
            config: create.config,
          };
        });
      }

      if (input.updates && input.updates.length > 0) {
        multiInput.updates = input.updates.map((update) => {
          const updateItem: any = {
            paramId: update.paramId,
          };
          if (update.name !== undefined) updateItem.name = update.name;
          if (update.defaultValue !== undefined)
            updateItem.defaultValue = update.defaultValue;
          if (update.type !== undefined) {
            const gqlType = toGqlParameterType(update.type);
            if (!gqlType)
              throw new Error(`Invalid parameter type: ${update.type}`);
            updateItem.type = gqlType;
          }
          if (update.required !== undefined)
            updateItem.required = update.required;
          if (update.publicValue !== undefined)
            updateItem.public = update.publicValue;
          if (update.config !== undefined) updateItem.config = update.config;
          return updateItem;
        });
      }

      if (input.deletes && input.deletes.length > 0) {
        multiInput.deletes = input.deletes;
      }

      if (input.reorders && input.reorders.length > 0) {
        multiInput.reorders = input.reorders.map((reorder) => ({
          paramId: reorder.paramId,
          newIndex: reorder.newIndex,
        }));
      }

      const data = await graphQLContext?.UpdateWorkflowVariables({
        input: multiInput,
      });

      if (data?.updateParameters) {
        return data.updateParameters.map((param) => toWorkflowVariable(param));
      }
      return [];
    },
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: [ParameterQueryKeys.GetParameters, variables.projectId],
      });
    },
  });

  const deleteWorkflowVariablesMutation = useMutation({
    mutationFn: async ({
      paramIds,
      projectId,
    }: {
      paramIds: string[];
      projectId: string;
    }) => {
      const data = await graphQLContext?.DeleteWorkflowVariables({
        input: {
          paramIds,
        },
      });
      if (data?.removeParameters === true) {
        return { success: true, projectId };
      }
      throw new Error("Failed to delete workflow variables");
    },
    onSuccess: (result) => {
      if (result?.success && result?.projectId) {
        queryClient.invalidateQueries({
          queryKey: [ParameterQueryKeys.GetParameters, result.projectId],
        });
      }
    },
  });

  return {
    useWorkflowVariablesQuery,
    createWorkflowVariablesMutation,
    updateMultipleWorkflowVariablesMutation,
    deleteWorkflowVariableMutation,
    deleteWorkflowVariablesMutation,
  };
};
