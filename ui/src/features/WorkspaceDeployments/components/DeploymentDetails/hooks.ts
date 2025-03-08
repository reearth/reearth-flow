import { useRouter } from "@tanstack/react-router";
import { useCallback, useMemo, useState } from "react";

import { DetailsBoxContent } from "@flow/features/common";
import { useT } from "@flow/lib/i18n";
import { Deployment } from "@flow/types";
import { formatTimestamp } from "@flow/utils";

export default ({ selectedDeployment }: { selectedDeployment: Deployment }) => {
  const t = useT();
  const { history } = useRouter();

  const [openDeploymentEditDialog, setOpenDeploymentEditDialog] =
    useState(false);

  const handleBack = useCallback(() => history.go(-1), [history]); // Go back to previous page

  const details: DetailsBoxContent[] | undefined = useMemo(
    () =>
      selectedDeployment
        ? [
            {
              id: "id",
              name: t("ID"),
              value:
                selectedDeployment.id || t("Unknown or deleted deployment"),
            },
            {
              id: "description",
              name: t("Description"),
              value: selectedDeployment.description || t("N/A"),
            },
            {
              id: "project",
              name: t("Project Name"),
              value:
                selectedDeployment.projectName ||
                t("Unknown or deleted project"),
            },
            {
              id: "version",
              name: t("Version"),
              value: selectedDeployment.version || "",
            },
            {
              id: "createdAt",
              name: t("Created At"),
              value:
                formatTimestamp(selectedDeployment.createdAt) ||
                t("Never") ||
                "",
            },
            {
              id: "updatedAt",
              name: t("Updated At"),
              value:
                formatTimestamp(selectedDeployment.updatedAt) ||
                t("Never") ||
                "",
            },
            {
              id: "workflowDownload",
              name: t("Workflow Download"),
              value: selectedDeployment.workflowUrl,
              type: "download",
            },
          ]
        : undefined,
    [t, selectedDeployment],
  );

  return {
    details,
    openDeploymentEditDialog,
    handleBack,
    setOpenDeploymentEditDialog,
  };
};
