import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogHeader,
  DialogTitle,
  Input,
  Label,
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
      onOpenChange={(o) => !o && setEditProject(undefined)}
    >
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("Edit Project")}</DialogTitle>
          <DialogContentWrapper>
            <DialogContentSection>
              <Label>{t("Project Name")}</Label>
              <Input
                value={editProject?.name}
                placeholder={t("Project Name")}
                onChange={(e) => onUpdateValue("name", e.target.value)}
              />
            </DialogContentSection>
            <DialogContentSection>
              <Label>{t("Project Description")}</Label>
              <Input
                placeholder={t("Project Description")}
                value={editProject?.description}
                onChange={(e) => onUpdateValue("description", e.target.value)}
              />
            </DialogContentSection>
            <div
              className={`text-xs text-red-400 ${showError ? "opacity-70" : "opacity-0"}`}
            >
              {t("Failed to update project")}
            </div>
          </DialogContentWrapper>

          <div className="flex justify-end gap-4 px-6 pb-6">
            {/* <Button
              disabled={buttonDisabled}
              variant={"outline"}
              onClick={() => setEditProject(undefined)}
            >
              {t("Cancel")}
            </Button> */}
            <Button
              disabled={buttonDisabled || !editProject?.name}
              onClick={onUpdateProject}
            >
              {t("Save")}
            </Button>
          </div>
        </DialogHeader>
      </DialogContent>
    </Dialog>
  );
};

export { ProjectEditDialog };
