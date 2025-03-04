import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  Label,
  DialogFooter,
  Input,
} from "@flow/components";
import { ALLOWED_WORKFLOW_FILE_EXTENSIONS } from "@flow/global-constants";
import { useT } from "@flow/lib/i18n";
import { Deployment } from "@flow/types";

import useHooks from "./hooks";

type Props = {
  selectedDeployment: Deployment;
  onDialogClose: () => void;
};

const DeploymentEditDialog: React.FC<Props> = ({
  selectedDeployment,
  onDialogClose,
}) => {
  const t = useT();

  const {
    workflowFile,
    invalidFile,
    updatedDescription,
    handleWorkflowFileUpload,
    handleDescriptionChange,
    handleDeploymentUpdate,
  } = useHooks({ selectedDeployment, onDialogClose });

  return (
    <Dialog open={true} onOpenChange={onDialogClose}>
      <DialogContent size="sm">
        <DialogTitle>{t("Edit Deployment")}</DialogTitle>
        <DialogContentWrapper>
          {!selectedDeployment.projectName && (
            <DialogContentSection className="flex flex-col">
              <Label>{t("Workflow file: ")}</Label>
              <Input
                type="file"
                accept={ALLOWED_WORKFLOW_FILE_EXTENSIONS}
                onChange={handleWorkflowFileUpload}
              />
              {invalidFile && (
                <p className="text-xs text-red-500 dark:text-red-400">
                  {t(
                    "There is a problem with file you tried to upload. Please verify its contents and try again.",
                  )}
                </p>
              )}
            </DialogContentSection>
          )}
          <div className="border-b border-primary text-center" />
          <DialogContentSection className="flex flex-col">
            <Label>{t("Description")}</Label>
            <Input
              value={updatedDescription}
              onChange={handleDescriptionChange}
              placeholder={t(
                "Give your deployment a meaningful description...",
              )}
            />
          </DialogContentSection>
          <DialogContentSection>
            <p className="dark:font-light">
              {t("Are you sure you want to proceed?")}
            </p>
          </DialogContentSection>
        </DialogContentWrapper>
        <DialogFooter>
          <Button
            disabled={
              (updatedDescription === selectedDeployment.description ||
                !updatedDescription.trim()) &&
              !workflowFile
            }
            onClick={handleDeploymentUpdate}>
            {t("Update Deployment")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export { DeploymentEditDialog };
