import { useNavigate } from "@tanstack/react-router";
import { useCallback } from "react";
import * as Y from "yjs";
import type { Doc } from "yjs";

import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useProjectImport } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";
import type { Project, Workspace } from "@flow/types";

import useSharedWorkflowVariables from "../../useSharedWorkflowVariables";

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
  const { workflowVariables } = useSharedWorkflowVariables();
  const handleSharedProjectImport = useCallback(async (): Promise<boolean> => {
    const showImportFailed = () =>
      toast({
        title: t("Project Import Failed"),
        description: t(
          "Project could not be imported into the selected workspace",
        ),
        variant: "destructive",
      });

    if (!sharedYdoc || !sharedProject || !accessToken || !selectedWorkspace) {
      console.error(
        "Missing either sharedYdoc, sharedProject, accessToken, or selectedWorkspace",
      );
      showImportFailed();
      return false;
    }

    const yDocBinary = Y.encodeStateAsUpdate(sharedYdoc);

    const importedProject = await handleProjectImport({
      yDocBinary,
      projectName: sharedProject.name,
      projectDescription: sharedProject.description,
      workspace: selectedWorkspace,
      workflowVariables,
    });

    if (!importedProject) {
      showImportFailed();
      return false;
    }

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
    navigate({
      to: `/workspaces/${selectedWorkspace.id}/projects/${importedProject.id}`,
    });
    return true;
  }, [
    accessToken,
    selectedWorkspace,
    sharedYdoc,
    sharedProject,
    workflowVariables,
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
