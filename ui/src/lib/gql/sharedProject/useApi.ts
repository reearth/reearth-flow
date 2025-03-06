import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import { ShareProject, UnshareProject } from "@flow/types";

import { ShareProjectInput, UnshareProjectInput } from "../__gen__/graphql";

import { useQueries } from "./useQueries";

export const useSharedProject = () => {
  const { toast } = useToast();
  const t = useT();

  const {
    shareProjectMutation,
    unshareProjectMutation,
    useGetSharedProjectQuery,
    useGetSharedProjectInfoQuery,
  } = useQueries();

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

  const unshareProject = async (
    input: UnshareProjectInput & { workspaceId: string },
  ): Promise<UnshareProject> => {
    const { mutateAsync, ...rest } = unshareProjectMutation;
    try {
      const unsharedProject = await mutateAsync(input);
      toast({
        title: t("Project Unshared"),
        description: t("Project has been successfully unshared."),
      });
      return { ...unsharedProject, ...rest };
    } catch (_err) {
      return { projectId: undefined, ...rest };
    }
  };

  const useGetSharedProject = (token?: string) => {
    const { data, ...rest } = useGetSharedProjectQuery(token);
    return {
      sharedProject: data?.sharedProject,
      ...rest,
    };
  };

  const useGetSharedProjectInfo = (projectId?: string) => {
    const { data, ...rest } = useGetSharedProjectInfoQuery(projectId);
    return {
      projectId: data?.projectSharingInfo.projectId,
      sharedToken: data?.projectSharingInfo.sharingToken,
      ...rest,
    };
  };

  return {
    shareProject,
    unshareProject,
    useGetSharedProject,
    useGetSharedProjectInfo,
  };
};
