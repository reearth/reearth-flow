import { useCallback, useState } from "react";

import { useProject, useWorkflowVariables } from "@flow/lib/gql";
import { toGqlProjectVisibility } from "@flow/lib/gql/convert";
import { useCurrentWorkspace } from "@flow/stores";
import type { Project, Workspace } from "@flow/types";

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
    async (project: Project, targetWorkspace?: Workspace): Promise<boolean> => {
      const workspace = targetWorkspace ?? currentWorkspace;
      if (!project || !workspace) {
        return false;
      }

      try {
        setIsDuplicating(true);

        // The copy takes the project's details but not its alias, which is
        // unique to it: the copy gets its own default alias.
        const { project: newProject } = await createProject({
          workspaceId: workspace.id,
          name: project.name,
          description: project.description,
          visibility: toGqlProjectVisibility(project.visibility),
          readme: project.readme || undefined,
          license: project.license || undefined,
          topics: project.topics.length ? project.topics : undefined,
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

        const { success } = await copyProject(
          newProject.id,
          project.id,
          workspace.id,
        );
        return !!success;
      } catch (error) {
        console.error("Project duplication failed:", error);
        return false;
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
