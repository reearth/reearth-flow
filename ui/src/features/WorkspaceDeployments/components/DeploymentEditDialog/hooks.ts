import { ChangeEvent, useCallback, useState } from "react";

import { useDeployment } from "@flow/lib/gql";
import { Deployment } from "@flow/types";
import { validateWorkflowYaml } from "@flow/utils/engineWorkflowValidation";
import { validateWorkflowJson } from "@flow/utils/engineWorkflowValidation/jsonValidation";

export default ({
  selectedDeployment,
  onDialogClose,
}: {
  selectedDeployment: Deployment;
  onDialogClose: () => void;
}) => {
  const { useUpdateDeployment } = useDeployment();

  const [updatedDescription, setUpdatedDescription] = useState(
    selectedDeployment?.description || "",
  );
  const [workflowFile, setWorkflowFile] = useState<File | undefined>(undefined);
  const [invalidFile, setInvalidFile] = useState<boolean>(false);

  const handleDescriptionChange = useCallback(
    (e: ChangeEvent<HTMLInputElement>) => {
      setUpdatedDescription(e.target.value);
    },
    [],
  );

  const handleWorkflowFileUpload = useCallback(
    (e: ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;

      const fileExtension = file.name.split(".").pop();
      if (fileExtension === "json") {
        const reader = new FileReader();

        reader.onload = (e2) => {
          const results = e2.target?.result;
          if (results && typeof results === "string") {
            if (validateWorkflowJson(results).isValid) {
              setInvalidFile(false);
            } else {
              setInvalidFile(true);
            }
            setWorkflowFile(e.target.files?.[0] || undefined);
          }
        };

        reader.onerror = (e) => {
          console.error("Error reading file:", e.target?.error);
        };

        // Read the file as text
        reader.readAsText(file);
      } else if (fileExtension === "yaml" || fileExtension === "yml") {
        const reader = new FileReader();

        reader.onload = (e2) => {
          const results = e2.target?.result;
          if (results && typeof results === "string") {
            if (validateWorkflowYaml(results).isValid) {
              setInvalidFile(false);
            } else {
              setInvalidFile(true);
            }
            setWorkflowFile(e.target.files?.[0] || undefined);
          }
        };

        reader.onerror = (e) => {
          console.error("Error reading file:", e.target?.error);
        };

        // Read the file as text
        reader.readAsText(file);
      }
    },
    [],
  );

  const handleDeploymentUpdate = useCallback(async () => {
    if (!selectedDeployment) return;

    await useUpdateDeployment(
      selectedDeployment.id,
      workflowFile,
      updatedDescription,
    );

    onDialogClose();
  }, [
    selectedDeployment,
    workflowFile,
    updatedDescription,
    onDialogClose,
    useUpdateDeployment,
  ]);

  return {
    workflowFile,
    invalidFile,
    updatedDescription,
    handleWorkflowFileUpload,
    handleDescriptionChange,
    handleDeploymentUpdate,
  };
};
