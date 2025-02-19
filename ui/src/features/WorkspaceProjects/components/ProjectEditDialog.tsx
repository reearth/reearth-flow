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
import { useT } from "@flow/lib/i18n";
import { Project } from "@flow/types";

type Props = {
  editProject: Project | undefined;
  showError: boolean;
  buttonDisabled: boolean;
  setEditProject: (project: Project | undefined) => void;
  onUpdateValue: (key: "name" | "description", value: string) => void;
  onUpdateProject: () => void;
};

const ProjectEditDialog: React.FC<Props> = ({
  editProject,
  showError,
  buttonDisabled,
  setEditProject,
  onUpdateValue,
  onUpdateProject,
}) => {
  const t = useT();
  return (
    <Dialog
      open={!!editProject}
      onOpenChange={(o) => !o && setEditProject(undefined)}>
      <DialogContent size="md">
        <DialogHeader>
          <DialogTitle>{t("Edit Project")}</DialogTitle>
        </DialogHeader>
        <DialogContentWrapper>
          <DialogContentSection>
            <Label>{t("Project Name")}</Label>
            <Input
              value={editProject?.name}
              placeholder={t("Your project name goes here...")}
              onChange={(e) => onUpdateValue("name", e.target.value)}
            />
          </DialogContentSection>
          <DialogContentSection>
            <Label>{t("Project Description")}</Label>
            <TextArea
              placeholder={t("Your project description goes here...")}
              value={editProject?.description}
              onChange={(e) => onUpdateValue("description", e.target.value)}
            />
          </DialogContentSection>
          <div
            className={`text-xs text-red-400 ${showError ? "opacity-70" : "opacity-0"}`}>
            {t("Failed to update project")}
          </div>
        </DialogContentWrapper>
        <DialogFooter>
          {/* <Button
              disabled={buttonDisabled}
              variant={"outline"}
              onClick={() => setEditProject(undefined)}
              >
              {t("Cancel")}
              </Button> */}
          <Button
            disabled={buttonDisabled || !editProject?.name}
            onClick={onUpdateProject}>
            {t("Save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export { ProjectEditDialog };
