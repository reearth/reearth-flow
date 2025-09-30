import { useCallback, useState } from "react";

import { useProject } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import type { Workspace } from "@flow/types";

export default () => {
  const [isProjectImporting, setIsProjectImporting] = useState<boolean>(false);
  const t = useT();

  const { createProject, importProject } = useProject();

  const handleProjectImport = useCallback(
    async ({
      projectName,
      projectDescription,
      workspace,
      yDocBinary,
    }: {
      projectName: string;
      projectDescription: string;
      workspace: Workspace;
      yDocBinary: Uint8Array<ArrayBufferLike>;
      accessToken: string;
    }) => {
      try {
        setIsProjectImporting(true);

        const { project } = await createProject({
          workspaceId: workspace.id,
          name: projectName + t("(import)"),
          description: projectDescription,
        });

        if (!project) {
          console.error("Failed to create project");
          return;
        }

        await importProject(project.id, yDocBinary, workspace.id);
      } catch (error) {
        console.error("Failed to import project:", error);
      } finally {
        setIsProjectImporting(false);
      }
    },
    [createProject, importProject, t],
  );

  return {
    isProjectImporting,
    handleProjectImport,
  };
};
