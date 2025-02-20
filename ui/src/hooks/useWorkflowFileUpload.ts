import { ChangeEvent, useCallback, useState } from "react";

import {
  validateWorkflowJson,
  validateWorkflowYaml,
} from "@flow/utils/engineWorkflowValidation";

export default () => {
  const [workflowFile, setWorkflowFile] = useState<File | null>(null);
  const [invalidFile, setInvalidFile] = useState<boolean>(false);

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
            setWorkflowFile(e.target.files?.[0] || null);
          }
        };

        reader.onerror = (e) => {
          console.error("Error reading file:", e.target?.error);
        };

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
            setWorkflowFile(e.target.files?.[0] || null);
          }
        };

        reader.onerror = (e) => {
          console.error("Error reading file:", e.target?.error);
        };

        reader.readAsText(file);
      }
    },
    [],
  );

  return {
    workflowFile,
    invalidFile,
    handleWorkflowFileUpload,
  };
};
