import { useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentWrapper,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@flow/components";
import { useProject } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import type { Project } from "@flow/types";

import { ProjectFormFields, useProjectForm } from "./ProjectForm";

type Props = {
  project: Project;
  onClose: () => void;
};

const ProjectEditDialog: React.FC<Props> = ({ project, onClose }) => {
  const t = useT();
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | undefined>(undefined);
  const { updateProject } = useProject();
  const form = useProjectForm({ workspaceId: project.workspaceId, project });

  const handleUpdateProject = async () => {
    if (!form.canSubmit) return;
    setError(undefined);
    setIsSubmitting(true);
    const { project: updated, errorMessage } = await updateProject(
      form.toUpdateInput(project),
    );
    setIsSubmitting(false);

    if (!updated) {
      setError(errorMessage ?? t("Failed to update project"));
      return;
    }
    onClose();
  };

  return (
    <Dialog open onOpenChange={(o) => !o && onClose()}>
      <DialogContent size="lg">
        <DialogHeader>
          <DialogTitle>{t("Edit Project")}</DialogTitle>
        </DialogHeader>
        <DialogContentWrapper>
          <ProjectFormFields form={form} projectId={project.id} />
          <div
            className={`text-xs text-red-400 ${error ? "opacity-70" : "opacity-0"}`}>
            {error ?? t("Failed to update project")}
          </div>
        </DialogContentWrapper>
        <DialogFooter>
          <Button
            disabled={!form.canSubmit || isSubmitting}
            onClick={handleUpdateProject}>
            {t("Save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export { ProjectEditDialog };
