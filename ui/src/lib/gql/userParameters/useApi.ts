import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import type { CreateUserParamater, VarType } from "@flow/types";

import { useQueries } from "./useQueries";

export const useUserParameter = () => {
  const { useUserParametersQuery, createUserParametersMutation } = useQueries();

  const { toast } = useToast();
  const t = useT();

  const useGetUserParameters = (projectId?: string) => {
    const { data, ...rest } = useUserParametersQuery(projectId);
    return {
      userParameters: data,
      ...rest,
    };
  };

  const createUserParameter = async (
    projectId: string,
    name: string,
    value: any,
    type: VarType,
    required: boolean,
    index: number,
  ): Promise<CreateUserParamater> => {
    const { mutateAsync, ...rest } = createUserParametersMutation;
    try {
      const userParameter = await mutateAsync({
        projectId,
        name,
        value,
        type,
        required,
        index,
      });

      toast({
        title: t("User Parameter Created"),
        description: t(
          "User parameter {{name}} has been created successfully.",
          { name },
        ),
      });

      return { userParameter, ...rest };
    } catch (_err) {
      toast({
        title: t("Project Rollback Failed"),
        description: t("There was an error rolling back the project."),
        variant: "warning",
      });

      return { userParameter: undefined, ...rest };
    }
  };

  return {
    useGetUserParameters,
    createUserParameter,
  };
};
