import { useCallback, useState } from "react";

import { useProject, useWorkflowVariables } from "@flow/lib/gql";
import { useCurrentWorkspace } from "@flow/stores";
import { Project } from "@flow/types";

export default (projectToDuplicate?: Project) => {
  const [isDuplicating, setIsDuplicating] = useState<boolean>(false);
  const [currentWorkspace] = useCurrentWorkspace();
  const { createProject, copyProject } = useProject();
  const { useGetWorkflowVariables, updateMultipleWorkflowVariables } =
    useWorkflowVariables();
  const { workflowVariables } = useGetWorkflowVariables(
    projectToDuplicate?.id ?? "",
  );

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

        if (workflowVariables && workflowVariables.length > 0 && newProject) {
          await updateMultipleWorkflowVariables({
            projectId: newProject.id,
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
    [
      currentWorkspace,
      workflowVariables,
      createProject,
      copyProject,
      updateMultipleWorkflowVariables,
    ],
  );

  return {
    isDuplicating,
    handleProjectDuplication,
  };
};
