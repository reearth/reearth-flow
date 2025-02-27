import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import { ShareProject } from "@flow/types";

import { ShareProjectInput } from "../__gen__/graphql";

import { useQueries } from "./useQueries";

export const useSharedProject = () => {
  const { toast } = useToast();
  const t = useT();

  const { shareProjectMutation, useGetSharedProjectQuery } = useQueries();

  const shareProject = async (
    input: ShareProjectInput & { workspaceId: string },
  ): Promise<ShareProject> => {
    const { mutateAsync, ...rest } = shareProjectMutation;
    try {
      const sharedProject = await mutateAsync(input);
      toast({
        title: t("Project Shared"),
        description: t("Project has been successfully shared."),
      });
      return { ...sharedProject, ...rest };
    } catch (_err) {
      return { projectId: undefined, sharingUrl: undefined, ...rest };
    }
  };

  const useGetSharedProject = (token?: string) => {
    const { data, ...rest } = useGetSharedProjectQuery(token);
    return {
      sharedProject: data?.sharedProject,
      ...rest,
    };
  };

  return {
    shareProject,
    useGetSharedProject,
  };
};
