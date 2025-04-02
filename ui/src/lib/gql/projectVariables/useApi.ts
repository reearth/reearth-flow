import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import type { CreateProjectVariables, VarType } from "@flow/types";

import { useQueries } from "./useQueries";

export const useProjectVariables = () => {
  const { useProjectVariablesQuery, createProjectVariablesMutation } =
    useQueries();

  const { toast } = useToast();
  const t = useT();

  const useGetProjectVariables = (projectId?: string) => {
    const { data, ...rest } = useProjectVariablesQuery(projectId);
    return {
      projectVariables: data,
      ...rest,
    };
  };

  const createProjectVariable = async (
    projectId: string,
    name: string,
    value: any,
    type: VarType,
    required: boolean,
    index: number,
  ): Promise<CreateProjectVariables> => {
    const { mutateAsync, ...rest } = createProjectVariablesMutation;
    try {
      const projectVariable = await mutateAsync({
        projectId,
        name,
        value,
        type,
        required,
        index,
      });

      toast({
        title: t("Project Variable Created"),
        description: t(
          "Project variable {{name}} has been created successfully.",
          { name },
        ),
      });

      return { projectVariable, ...rest };
    } catch (_err) {
      toast({
        title: t("Project Variable Creation Failed"),
        description: t("There was an error creating a project variable."),
        variant: "warning",
      });

      return { projectVariable: undefined, ...rest };
    }
  };

  return {
    useGetProjectVariables,
    createProjectVariable,
  };
};
