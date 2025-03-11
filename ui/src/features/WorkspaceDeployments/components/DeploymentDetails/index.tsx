import { CaretLeft, PencilLine, Play, Trash } from "@phosphor-icons/react";

import { Button } from "@flow/components";
import { DetailsBox } from "@flow/features/common";
import { useT } from "@flow/lib/i18n";
import { Deployment } from "@flow/types";

import { DeploymentEditDialog } from "../DeploymentEditDialog";

import useHooks from "./hooks";

type Props = {
  selectedDeployment: Deployment;
  setDeploymentToBeDeleted: (deployment?: Deployment) => void;
  onDeploymentRun: (deployment?: Deployment) => Promise<void>;
};

const DeploymentDetails: React.FC<Props> = ({
  selectedDeployment,
  setDeploymentToBeDeleted,
  onDeploymentRun,
}) => {
  const t = useT();

  const {
    details,
    openDeploymentEditDialog,
    handleBack,
    setOpenDeploymentEditDialog,
  } = useHooks({ selectedDeployment });

  return (
    <>
      <div className="flex flex-1 flex-col gap-4 px-6 pb-2 pt-6">
        <div className="flex justify-between">
          <Button size="icon" variant="ghost" onClick={handleBack}>
            <CaretLeft />
          </Button>
          <div className="flex gap-2">
            <Button
              variant="default"
              size="sm"
              onClick={() => onDeploymentRun(selectedDeployment)}>
              <Play />
              {t("Run")}
            </Button>
            <Button
              variant="outline"
              size="sm"
              disabled={!selectedDeployment}
              onClick={() => setOpenDeploymentEditDialog(true)}>
              <PencilLine />
              {t("Edit Deployment")}
            </Button>
            <Button
              variant="destructive"
              size="sm"
              onClick={() => setDeploymentToBeDeleted(selectedDeployment)}>
              <Trash />
              {t("Delete")}
            </Button>
          </div>
        </div>
        <div className="w-full border-b" />
        <div className="mt-6 flex max-w-[1200px] flex-col gap-6">
          <DetailsBox title={t("Deployment Details")} content={details} />
        </div>
      </div>
      {openDeploymentEditDialog && selectedDeployment && (
        <DeploymentEditDialog
          selectedDeployment={selectedDeployment}
          onDialogClose={() => setOpenDeploymentEditDialog(false)}
        />
      )}
    </>
  );
};

export { DeploymentDetails };
