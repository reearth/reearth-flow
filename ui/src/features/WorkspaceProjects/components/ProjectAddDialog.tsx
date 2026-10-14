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
import { useCurrentWorkspace } from "@flow/stores";

import { ProjectFormFields, useProjectForm } from "./ProjectForm";

type Props = {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
};

const ProjectAddDialog: React.FC<Props> = ({ isOpen, onOpenChange }) => {
  const t = useT();
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | undefined>(undefined);
  const { createProject } = useProject();
  const [currentWorkspace] = useCurrentWorkspace();
  const form = useProjectForm({ workspaceId: currentWorkspace?.id });

  const handleOpenChange = (open: boolean) => {
    if (!open) {
      form.reset();
      setError(undefined);
    }
    onOpenChange(open);
  };

  const handleClick = async () => {
    if (!form.canSubmit || !currentWorkspace) return;
    setError(undefined);
    setIsSubmitting(true);
    const { project, errorMessage } = await createProject(
      form.toCreateInput(currentWorkspace.id),
    );
    setIsSubmitting(false);

    if (!project) {
      setError(errorMessage ?? t("Failed to create project"));
      return;
    }
    handleOpenChange(false);
  };

  return (
    <Dialog open={isOpen} onOpenChange={handleOpenChange}>
      <DialogContent size="lg" position="off-center">
        <DialogHeader>
          <DialogTitle>{t("New Project")}</DialogTitle>
        </DialogHeader>
        <DialogContentWrapper>
          <ProjectFormFields form={form} />
          <div
            className={`text-xs text-red-400 ${error ? "opacity-70" : "opacity-0"}`}>
            {error ?? t("Failed to create project")}
          </div>
        </DialogContentWrapper>
        <DialogFooter>
          <Button
            className="self-end"
            disabled={!form.canSubmit || isSubmitting}
            size="sm"
            onClick={handleClick}>
            {t("Create")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export { ProjectAddDialog };
