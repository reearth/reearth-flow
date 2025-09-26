import { useCallback, useState } from "react";

import { useProject } from "@flow/lib/gql";
import { useCurrentWorkspace } from "@flow/stores";
import { Project } from "@flow/types";

export default () => {
  const [isDuplicating, setIsDuplicating] = useState<boolean>(false);
  const [currentWorkspace] = useCurrentWorkspace();
  const { createProject, copyProject } = useProject();

  const handleProjectDuplication = useCallback(
    async (project: Project) => {
      if (!project || !currentWorkspace) {
        return;
      }

      try {
        setIsDuplicating(true);

        const { project: newProject } = await createProject({
          workspaceId: currentWorkspace.id,
          name: project.name,
          description: project.description,
        });

        if (!newProject) {
          throw new Error("Failed to create new project");
        }

        await copyProject(newProject.id, project.id, currentWorkspace.id);
      } catch (error) {
        console.error("Project duplication failed:", error);
      } finally {
        setIsDuplicating(false);
      }
    },
    [currentWorkspace, createProject, copyProject],
  );

  return {
    isDuplicating,
    handleProjectDuplication,
  };
};
