import { CaretRight } from "@phosphor-icons/react";
import { DialogDescription } from "@radix-ui/react-dialog";
import { Dispatch, SetStateAction } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  Label,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  setShowDialog: Dispatch<SetStateAction<"deploy" | undefined>>;
};

const DeployDialog: React.FC<Props> = ({ setShowDialog }) => {
  const t = useT();

  return (
    <Dialog open={true} onOpenChange={() => setShowDialog(undefined)}>
      <DialogContent size="sm">
        <DialogTitle>{t("Deploy project workflow")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection>
            <Label>{t("Project to deploy: ")}</Label>
            <p className="font-thin">My Project</p>
          </DialogContentSection>
          <DialogContentSection>
            <Label>{t("Deploy version: ")}</Label>
            <div className="flex items-center">
              <p className="font-thin">1.0</p>
              <CaretRight />
              <p className="font-semibold">2.0</p>
            </div>
          </DialogContentSection>
          <DialogDescription>
            {t("Are you sure you want to proceed?")}
          </DialogDescription>
        </DialogContentWrapper>
        <div className="flex justify-end gap-4 px-6 pb-6">
          <Button
          // disabled={buttonDisabled || !editProject?.name}
          // onClick={onUpdateProject}
          >
            {t("Deploy")}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default DeployDialog;
