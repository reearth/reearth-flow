import { CaretRight } from "@phosphor-icons/react";
import { Dispatch, SetStateAction } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  Label,
  DialogFooter,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { useCurrentProject } from "@flow/stores";

type Props = {
  onWorkflowDeployment: () => void;
  setShowDialog: Dispatch<SetStateAction<"deploy" | undefined>>;
};

const DeployDialog: React.FC<Props> = ({
  onWorkflowDeployment,
  setShowDialog,
}) => {
  const t = useT();
  const [currentProject] = useCurrentProject();

  return (
    <Dialog open={true} onOpenChange={() => setShowDialog(undefined)}>
      <DialogContent size="sm">
        <DialogTitle>{t("Deploy project workflow")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection className="flex flex-col">
            <Label>{t("Project to deploy: ")}</Label>
            <p className="truncate dark:font-thin">
              {currentProject?.name ?? t("N/A")}
            </p>
          </DialogContentSection>
          <DialogContentSection>
            <Label>{t("Deploy version: ")}</Label>
            <div className="flex items-center">
              <p className="dark:font-thin">1.0</p>
              <CaretRight />
              <p className="font-semibold">2.0</p>
            </div>
          </DialogContentSection>
          <DialogContentSection>
            <p className="dark:font-light">
              {t("Are you sure you want to proceed?")}
            </p>
          </DialogContentSection>
        </DialogContentWrapper>
        <DialogFooter>
          <Button
            // disabled={buttonDisabled || !editProject?.name}
            onClick={onWorkflowDeployment}>
            {t("Deploy")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default DeployDialog;
