import { useCallback, useState } from "react";

import { useProject, useWorkflowVariables } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import type { AnyWorkflowVariable, Workspace } from "@flow/types";

export default () => {
  const [isProjectImporting, setIsProjectImporting] = useState<boolean>(false);
  const t = useT();

  const { createProject, importProject } = useProject();
  const { updateMultipleWorkflowVariables } = useWorkflowVariables();

  const handleProjectImport = useCallback(
    async ({
      projectName,
      projectDescription,
      workspace,
      yDocBinary,
      workflowVariables,
    }: {
      projectName: string;
      projectDescription: string;
      workspace: Workspace;
      yDocBinary: Uint8Array<ArrayBufferLike>;
      workflowVariables?: AnyWorkflowVariable[];
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

        if (workflowVariables && workflowVariables.length > 0) {
          await updateMultipleWorkflowVariables({
            projectId: project.id,
            creates: workflowVariables.map((pv, index) => ({
              name: pv.name,
              defaultValue: pv.defaultValue,
              type: pv.type,
              required: pv.required,
              publicValue: pv.public,
              index,
              config: pv.config,
            })),
          });
        }

        await importProject(project.id, yDocBinary, workspace.id);
      } catch (error) {
        console.error("Failed to import project:", error);
      } finally {
        setIsProjectImporting(false);
      }
    },
    [createProject, importProject, updateMultipleWorkflowVariables, t],
  );

  return {
    isProjectImporting,
    handleProjectImport,
  };
};
