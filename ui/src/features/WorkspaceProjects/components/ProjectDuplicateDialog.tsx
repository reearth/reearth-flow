import { useMemo, useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  Label,
  TextArea,
} from "@flow/components";
import {
  WorkspaceSelect,
  getProjectCreatableWorkspaces,
} from "@flow/features/common";
import { useUser } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import type { Project, Workspace } from "@flow/types";

type Props = {
  duplicateProject: Project;
  setDuplicateProject: (project: Project | undefined) => void;
  onProjectDuplication: (
    project: Project,
    targetWorkspace: Workspace,
  ) => Promise<void>;
};

const ProjectDuplicateDialog: React.FC<Props> = ({
  duplicateProject,
  setDuplicateProject,
  onProjectDuplication,
}) => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  const { useGetMeAndWorkspaces } = useUser();
  const { me, workspaces } = useGetMeAndWorkspaces();

  const targetWorkspaces = useMemo(
    () => getProjectCreatableWorkspaces(workspaces, me?.id),
    [workspaces, me?.id],
  );

  const [pickedWorkspaceId, setPickedWorkspaceId] = useState<string>();
  const targetWorkspace = targetWorkspaces.find(
    (w) => w.id === (pickedWorkspaceId ?? currentWorkspace?.id),
  );

  const [name, setName] = useState(
    `${duplicateProject.name} ${t("(duplicate)")}`,
  );
  const [description, setDescription] = useState(duplicateProject.name);
  const handleProjectDuplication = async (
    name: string,
    description: string,
  ) => {
    if (!name || !targetWorkspace) return;
    if (!description) {
      setDescription("");
    }
    await onProjectDuplication(
      {
        ...duplicateProject,
        name,
        description,
      },
      targetWorkspace,
    );
    setDuplicateProject(undefined);
  };

  return (
    <Dialog
      open={!!duplicateProject}
      onOpenChange={(o) => !o && setDuplicateProject(undefined)}>
      <DialogContent size="md">
        <DialogHeader>
          <DialogTitle>{t("Duplicate Project")}</DialogTitle>
        </DialogHeader>
        <DialogContentWrapper>
          <DialogContentSection>
            <Label>{t("Workspace")}</Label>
            <WorkspaceSelect
              workspaces={targetWorkspaces}
              selectedWorkspaceId={targetWorkspace?.id}
              onSelectWorkspace={(w) => setPickedWorkspaceId(w.id)}
            />
          </DialogContentSection>
          <DialogContentSection>
            <Label>{t("Project Name")}</Label>
            <Input
              value={name}
              placeholder={t("Your project name goes here...")}
              onChange={(e) => setName(e.target.value)}
            />
          </DialogContentSection>
          <DialogContentSection>
            <Label>{t("Project Description")}</Label>
            <TextArea
              placeholder={t("Your project description goes here...")}
              value={description}
              onChange={(e) => setDescription(e.target.value)}
            />
          </DialogContentSection>
        </DialogContentWrapper>
        <DialogFooter>
          <Button
            variant={"outline"}
            onClick={() => setDuplicateProject(undefined)}>
            {t("Cancel")}
          </Button>
          <Button
            disabled={!name.trim() || !targetWorkspace}
            onClick={() => handleProjectDuplication(name, description)}>
            {t("Duplicate")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export { ProjectDuplicateDialog };
