import { useNavigate } from "@tanstack/react-router";
import { useCallback } from "react";
import * as Y from "yjs";
import { Doc } from "yjs";

import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useProjectImport } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";
import type { Project, Workspace } from "@flow/types";

type Props = {
  sharedYdoc: Doc | null;
  sharedProject?: Project;
  selectedWorkspace: Workspace | null;
  accessToken?: string;
};
export default ({
  sharedYdoc,
  sharedProject,
  selectedWorkspace,
  accessToken,
}: Props) => {
  const t = useT();
  const { toast } = useToast();

  const navigate = useNavigate();

  const { isProjectImporting, handleProjectImport } = useProjectImport();

  const handleSharedProjectImport = useCallback(async () => {
    if (!sharedYdoc || !sharedProject || !accessToken || !selectedWorkspace) {
      console.error(
        "Missing either sharedYdoc, sharedProject, accessToken, or selectedWorkspace",
      );
      toast({
        title: t("Project Import Failed"),
        description: t(
          "Project could not be imported into the selected workspace",
        ),
      });
      return;
    }

    const yDocBinary = Y.encodeStateAsUpdate(sharedYdoc);

    try {
      await handleProjectImport({
        yDocBinary,
        projectName: sharedProject.name,
        projectDescription: sharedProject.description,
        workspace: selectedWorkspace,
        accessToken,
      });
      toast({
        title: t("Project Imported"),
        description: t(
          "{{project}} has successfully been imported into {{workspace}}",
          {
            project: sharedProject.name,
            workspace: selectedWorkspace.name,
          },
        ),
      });
      navigate({ to: `/workspaces/${selectedWorkspace.id}/projects` });
    } catch (error) {
      console.error("Failed to import shared project:", error);
      toast({
        title: t("Project Import Failed"),
        description: t(
          "Project could not be imported into the selected workspace",
        ),
      });
    }
  }, [
    sharedYdoc,
    sharedProject,
    selectedWorkspace,
    accessToken,
    t,
    navigate,
    toast,
    handleProjectImport,
  ]);

  return {
    selectedWorkspace,
    isProjectImporting,
    handleSharedProjectImport,
  };
};
