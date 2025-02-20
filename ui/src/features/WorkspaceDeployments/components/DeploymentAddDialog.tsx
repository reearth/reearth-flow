import { useCallback, useState } from "react";

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
import useWorkflowFileUpload from "@flow/hooks/useWorkflowFileUpload";
import { useDeployment } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";

type Props = {
  setShowDialog: (show: boolean) => void;
};

const DeploymentAddDialog: React.FC<Props> = ({ setShowDialog }) => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  const { createDeploymentFromFile } = useDeployment();

  const [description, setDescription] = useState<string>("");

  const { workflowFile, invalidFile, handleWorkflowFileUpload } =
    useWorkflowFileUpload();

  const handleWorkflowDeployment = useCallback(async () => {
    const workspaceId = currentWorkspace?.id;

    if (!workspaceId || !workflowFile) return;

    await createDeploymentFromFile(workspaceId, workflowFile, description);

    setShowDialog(false);
  }, [
    currentWorkspace?.id,
    description,
    workflowFile,
    setShowDialog,
    createDeploymentFromFile,
  ]);

  return (
    <Dialog open={true} onOpenChange={() => setShowDialog(false)}>
      <DialogContent size="sm">
        <DialogTitle>{t("Create a deployment from file")}</DialogTitle>
        <DialogContentWrapper>
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
          <div className="border-b border-primary text-center" />
          <DialogContentSection className="flex flex-col">
            <Label>{t("Description")}</Label>
            <Input
              value={description}
              onChange={(e) => setDescription(e.target.value)}
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
            disabled={invalidFile || !workflowFile || !description.trim()}
            onClick={handleWorkflowDeployment}>
            {t("Deploy")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export { DeploymentAddDialog };
