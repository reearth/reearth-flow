import { ChangeEvent, useCallback, useState } from "react";

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
import { useDeployment } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import { validateWorkflowJson } from "@flow/utils/engineWorkflowValidation";
import { removeWhiteSpace } from "@flow/utils/removeWhiteSpace";

type Props = {
  setShowDialog: (show: boolean) => void;
};

const DeploymentAddDialog: React.FC<Props> = ({ setShowDialog }) => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  const { createDeploymentFromFile } = useDeployment();

  const [name, setName] = useState<string>("");

  const [description, setDescription] = useState<string>("");

  const [workflowFile, setWorkflowFile] = useState<File | null>(null);
  const [invalidFile, setInvalidFile] = useState<boolean>(false);

  const handleNameChange = useCallback((e: ChangeEvent<HTMLInputElement>) => {
    const trimmedValue = removeWhiteSpace(e.target.value);
    setName(trimmedValue);
  }, []);

  const handleWorkflowFileUpload = useCallback(
    (e: ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;
      const reader = new FileReader();

      reader.onload = (e2) => {
        const results = e2.target?.result;
        if (results && typeof results === "string") {
          if (validateWorkflowJson(results).isValid) {
            setInvalidFile(false);
          } else {
            setInvalidFile(true);
          }
          setWorkflowFile(e.target.files?.[0] || null);
        }
      };

      reader.onerror = (e) => {
        console.error("Error reading file:", e.target?.error);
      };

      // Read the file as text
      reader.readAsText(file);
    },
    [],
  );

  const handleWorkflowDeployment = useCallback(async () => {
    const workspaceId = currentWorkspace?.id;
    // const {
    //   name: projectName,
    //   workspaceId,
    //   id: projectId,
    // } = currentProject ?? {};

    if (!workspaceId || !workflowFile) return;
    // if (!workspaceId || !projectId) return;

    await createDeploymentFromFile(
      workspaceId,
      workflowFile,
      name,
      description,
    );

    setShowDialog(false);
  }, [
    currentWorkspace?.id,
    name,
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
              accept=".json"
              onChange={handleWorkflowFileUpload}
              placeholder={t("Give your deployment a unique name...")}
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
            <Label>{t("Name (optional): ")}</Label>
            <Input
              value={name}
              onChange={handleNameChange}
              placeholder={t("Give your deployment a unique name...")}
            />
          </DialogContentSection>
          <DialogContentSection className="flex flex-col">
            <Label>{t("Description (optional): ")}</Label>
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
            disabled={invalidFile || !workflowFile}
            onClick={handleWorkflowDeployment}>
            {t("Deploy")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export { DeploymentAddDialog };
