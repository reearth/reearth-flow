import { useMutation, useQuery } from "@tanstack/react-query";

import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import { RollbackProject } from "@flow/types";

import { toProjectDocument, toProjectSnapShot, toProjectSnapShotMeta } from "../convert";
import { useDocumentGraphQLContext } from "../provider/DocumentGraphQLProvider";

export const useDocument = () => {
  const sdk = useDocumentGraphQLContext();
  const { toast } = useToast();
  const t = useT();

  const useGetLatestProjectSnapshot = (projectId: string) => {
    const { data, ...rest } = useQuery({
      queryKey: ["latestProjectSnapshot", projectId],
      queryFn: async () => {
        if (!sdk) throw new Error("GraphQL SDK not available");
        const res = await sdk.GetLatestProjectSnapshot({ projectId });
        if (!res?.latestProjectSnapshot) return;
        return toProjectDocument(res.latestProjectSnapshot);
      },
      enabled: !!sdk && !!projectId,
    });
    
    return {
      projectDocument: data,
      ...rest,
    };
  };

  const useGetProjectHistory = (projectId: string) => {
    const { data, ...rest } = useQuery({
      queryKey: ["projectHistory", projectId],
      queryFn: async () => {
        if (!sdk) throw new Error("GraphQL SDK not available");
        const res = await sdk.GetProjectHistory({ projectId });
        if (!res) return;
        const { projectHistory } = res;
        return projectHistory.map((snapshot) => toProjectSnapShotMeta(snapshot));
      },
      enabled: !!sdk && !!projectId,
      refetchOnMount: false,
      refetchOnWindowFocus: false,
    });
    
    return {
      history: data,
      ...rest,
    };
  };

  const useGetProjectSnapshot = (projectId: string, version: number) => {
    const { data, ...rest } = useQuery({
      queryKey: ["projectSnapshot", projectId, version],
      queryFn: async () => {
        if (!sdk) throw new Error("GraphQL SDK not available");
        const res = await sdk.GetProjectSnapshot({ projectId, version });
        if (!res?.projectSnapshot) return;
        return toProjectSnapShot(res.projectSnapshot);
      },
      enabled: !!sdk && !!projectId && version > 0,
    });
    
    return {
      projectSnapshot: data,
      ...rest,
    };
  };

  const rollbackProjectMutation = useMutation({
    mutationFn: async ({ projectId, version }: { projectId: string; version: number }) => {
      if (!sdk) throw new Error("GraphQL SDK not available");
      const res = await sdk.RollbackProject({ projectId, version });
      return res.rollbackProject;
    },
  });

  const previewSnapshotMutation = useMutation({
    mutationFn: async ({ projectId, version }: { projectId: string; version: number }) => {
      if (!sdk) throw new Error("GraphQL SDK not available");
      const res = await sdk.PreviewSnapshot({ projectId, version });
      return res.previewSnapshot;
    },
  });

  const useRollbackProject = async (
    projectId: string,
    version: number,
  ): Promise<RollbackProject> => {
    const { mutateAsync, ...rest } = rollbackProjectMutation;
    try {
      const projectDocument = await mutateAsync({
        projectId,
        version,
      });

      toast({
        title: t("Project Rolled Back"),
        description: t(
          "Project has been successfully rolled back to version {{version}}.",
          { version },
        ),
      });

      return { projectDocument: projectDocument || undefined, ...rest };
    } catch (_err) {
      toast({
        title: t("Project Rollback Failed"),
        description: t("There was an error rolling back the project."),
        variant: "warning",
      });

      return { projectDocument: undefined, ...rest };
    }
  };

  const usePreviewSnapshot = async (projectId: string, version: number) => {
    const { mutateAsync, ...rest } = previewSnapshotMutation;
    try {
      const previewSnapshot = await mutateAsync({
        projectId,
        version,
      });

      return { previewSnapshot, ...rest };
    } catch (_err) {
      return { previewSnapshot: undefined, ...rest };
    }
  };

  return {
    useGetLatestProjectSnapshot,
    useGetProjectHistory,
    useGetProjectSnapshot,
    useRollbackProject,
    usePreviewSnapshot,
  };
}; 