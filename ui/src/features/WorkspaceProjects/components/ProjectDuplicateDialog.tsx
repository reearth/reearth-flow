import { useState } from "react";

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
// import ConfirmationDialog from "@flow/features/ConfirmationDialog";
import { useDocument } from "@flow/lib/gql/document";
import { useT } from "@flow/lib/i18n";
import { Project, ProjectDocument } from "@flow/types";

type Props = {
  duplicateProject: Project;
  setDuplicateProject: (project: Project | undefined) => void;
  onProjectDuplication: (
    project: Project,
    projectDocument?: ProjectDocument,
  ) => Promise<void>;
};

const ProjectDuplicateDialog: React.FC<Props> = ({
  duplicateProject,
  setDuplicateProject,
  onProjectDuplication,
}) => {
  const t = useT();
  const { useGetLatestProjectSnapshot } = useDocument();

  const { projectDocument } = useGetLatestProjectSnapshot(duplicateProject.id);
  const [name, setName] = useState(
    `${duplicateProject.name} ${t("(duplicate)")}`,
  );
  const [description, setDescription] = useState(duplicateProject.name);
  const handleProjectDuplication = async (
    name: string,
    description: string,
  ) => {
    if (!name) return;
    if (!description) {
      setDescription("");
    }
    await onProjectDuplication(
      {
        ...duplicateProject,
        name,
        description,
      },
      projectDocument,
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
            disabled={!name.trim()}
            onClick={() => handleProjectDuplication(name, description)}>
            {t("Duplicate")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export { ProjectDuplicateDialog };
