import { useCallback, useMemo, useState } from "react";

import { useProjectVariables } from "@flow/lib/gql";
import { useCurrentProject } from "@flow/stores";
import {
  ProjectVariable as ProjectVariableType,
  AnyProjectVariable,
} from "@flow/types";

export type DialogOptions =
  | "deploy"
  | "share"
  | "version"
  | "assets"
  | "debugStop"
  | "projectVariables"
  | undefined;

export default () => {
  const [showDialog, setShowDialog] = useState<DialogOptions>(undefined);
  const handleDialogOpen = (dialog: DialogOptions) => setShowDialog(dialog);
  const handleDialogClose = () => setShowDialog(undefined);

  const {
    useGetProjectVariables,
    createProjectVariable,
    updateMultipleProjectVariables,
    deleteProjectVariable,
    deleteProjectVariables,
  } = useProjectVariables();
  const [currentProject] = useCurrentProject();
  const { projectVariables } = useGetProjectVariables(currentProject?.id);
  const currentProjectVariables = useMemo(
    () => projectVariables ?? [],
    [projectVariables],
  );

  const handleProjectVariableAdd = useCallback(
    async (projectVariable: ProjectVariableType) => {
      if (!currentProject) return;

      await createProjectVariable(
        currentProject.id,
        projectVariable.name,
        projectVariable.defaultValue,
        projectVariable.type,
        projectVariable.required,
        projectVariable.public,
        currentProjectVariables.length,
        projectVariable.config,
      );
    },
    [currentProject, createProjectVariable, currentProjectVariables.length],
  );

  const handleProjectVariableChange = useCallback(
    async (projectVariable: ProjectVariableType) => {
      if (!currentProject) return;

      await updateMultipleProjectVariables({
        projectId: currentProject.id,
        updates: [
          {
            paramId: projectVariable.id,
            name: projectVariable.name,
            defaultValue: projectVariable.defaultValue,
            type: projectVariable.type,
            required: projectVariable.required,
            publicValue: projectVariable.public,
            config: projectVariable.config,
          },
        ],
      });
    },
    [updateMultipleProjectVariables, currentProject],
  );

  const handleProjectVariablesBatchUpdate = useCallback(
    async (input: {
      projectId: string;
      creates?: {
        name: string;
        defaultValue: any;
        type: ProjectVariableType["type"];
        required: boolean;
        publicValue: boolean;
        index?: number;
        config?: AnyProjectVariable["config"];
      }[];
      updates?: {
        paramId: string;
        name?: string;
        defaultValue?: any;
        type?: ProjectVariableType["type"];
        required?: boolean;
        publicValue?: boolean;
        config?: AnyProjectVariable["config"];
      }[];
      deletes?: string[];
    }) => {
      await updateMultipleProjectVariables(input);
    },
    [updateMultipleProjectVariables],
  );

  const handleProjectVariableDelete = useCallback(
    async (id: string) => {
      if (!currentProject) return;

      try {
        await deleteProjectVariable(id, currentProject.id);
      } catch (error) {
        console.error("Failed to delete project variable:", error);
      }
    },
    [deleteProjectVariable, currentProject],
  );

  const handleProjectVariablesBatchDelete = useCallback(
    async (ids: string[]) => {
      if (!currentProject) return;

      try {
        await deleteProjectVariables(currentProject.id, ids);
      } catch (error) {
        console.error("Failed to delete project variables:", error);
      }
    },
    [deleteProjectVariables, currentProject],
  );

  return {
    showDialog,
    currentProjectVariables,
    handleProjectVariableAdd,
    handleProjectVariableChange,
    handleProjectVariablesBatchUpdate,
    handleProjectVariableDelete,
    handleProjectVariablesBatchDelete,
    handleDialogOpen,
    handleDialogClose,
  };
};
