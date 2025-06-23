import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import type { ProjectVariable, VarType } from "@flow/types";
import { isDefined } from "@flow/utils";

import { toGqlParameterType, toProjectVariable } from "../convert";
import { useGraphQLContext } from "../provider";

export enum ParameterQueryKeys {
  GetParameters = "getParameters",
}

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const useProjectVariablesQuery = (projectId?: string) =>
    useQuery({
      queryKey: [ParameterQueryKeys.GetParameters, projectId],
      queryFn: async () => {
        const data = await graphQLContext?.GetProjectParameters({
          projectId: projectId ?? "",
        });

        if (!data) return;
        const { parameters } = data;
        const projectVars: ProjectVariable[] = parameters
          .filter(isDefined)
          .map((p) => toProjectVariable(p));

        return projectVars;
      },
      enabled: !!projectId,
      refetchOnMount: false,
      refetchOnWindowFocus: false,
    });

  const createProjectVariablesMutation = useMutation({
    mutationFn: async ({
      projectId,
      name,
      defaultValue,
      type,
      required,
      publicValue,
      index,
    }: {
      projectId: string;
      name: string;
      defaultValue: any;
      type: VarType;
      required: boolean;
      publicValue: boolean;
      index: number;
    }) => {
      const gqlType = toGqlParameterType(type);
      if (!gqlType) return;
      const data = await graphQLContext?.CreateProjectVariable({
        projectId,
        input: {
          name,
          defaultValue,
          type: gqlType,
          required,
          public: publicValue,
          index,
        },
      });

      if (data?.declareParameter) {
        return toProjectVariable(data?.declareParameter);
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

  const updateProjectVariablesMutation = useMutation({
    mutationFn: async ({
      paramId,
      name,
      defaultValue,
      type,
      required,
      publicValue,
    }: {
      paramId: string;
      name: string;
      defaultValue: any;
      type: VarType;
      required: boolean;
      publicValue: boolean;
    }) => {
      const gqlType = toGqlParameterType(type);
      if (!gqlType) return;
      const data = await graphQLContext?.UpdateProjectVariable({
        paramId,
        input: {
          name,
          defaultValue,
          type: gqlType,
          required,
          public: publicValue,
        },
      });
      if (data?.updateParameter) {
        return toProjectVariable(data?.updateParameter);
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

  const deleteProjectVariableMutation = useMutation({
    mutationFn: async ({
      paramId,
      projectId,
    }: {
      paramId: string;
      projectId: string;
    }) => {
      const data = await graphQLContext?.DeleteProjectVariable({
        input: {
          paramId,
        },
      });
      if (data?.removeParameter === true) {
        return { success: true, projectId };
      }
      throw new Error("Failed to delete project variable");
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

  const deleteProjectVariablesMutation = useMutation({
    mutationFn: async ({
      paramIds,
      projectId,
    }: {
      paramIds: string[];
      projectId: string;
    }) => {
      const data = await graphQLContext?.DeleteProjectVariables({
        input: {
          paramIds,
        },
      });
      if (data?.removeParameters === true) {
        return { success: true, projectId };
      }
      throw new Error("Failed to delete project variables");
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
    useProjectVariablesQuery,
    createProjectVariablesMutation,
    updateProjectVariablesMutation,
    deleteProjectVariableMutation,
    deleteProjectVariablesMutation,
  };
};
