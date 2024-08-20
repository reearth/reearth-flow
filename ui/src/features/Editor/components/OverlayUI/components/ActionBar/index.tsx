import {
  CaretRight,
  DownloadSimple,
  Play,
  RocketLaunch,
  Stop,
} from "@phosphor-icons/react";
import { DialogDescription } from "@radix-ui/react-dialog";
import { memo, useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  IconButton,
  Label,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

const tooltipOffset = 6;

const ActionBar = () => {
  const t = useT();

  const [showDialog, setShowDialog] = useState<"deploy" | undefined>(undefined);

  return (
    <>
      <div className="absolute right-1 top-1">
        <div className="m-1 rounded-md border bg-secondary">
          <div className="flex rounded-md">
            <div className="flex align-middle">
              {/* <IconButton
            tooltipText={t("Publish workflow")}
            tooltipOffset={tooltipOffset}
            icon={<DoubleArrowRightIcon />}
            /> */}
              <IconButton
                className="rounded-[4px]"
                tooltipText={t("Run project workflow")}
                tooltipOffset={tooltipOffset}
                icon={<Play weight="thin" />}
              />
              <IconButton
                className="rounded-[4px]"
                tooltipText={t("Stop project workflow")}
                tooltipOffset={tooltipOffset}
                icon={<Stop weight="thin" />}
              />
              <IconButton
                className="rounded-[4px]"
                tooltipText={t("Deploy project workflow")}
                tooltipOffset={tooltipOffset}
                icon={<RocketLaunch weight="thin" />}
                onClick={() => setShowDialog("deploy")}
              />
              <IconButton
                className="rounded-[4px]"
                tooltipText={t("Download project workflow")}
                tooltipOffset={tooltipOffset}
                icon={<DownloadSimple weight="thin" />}
              />
            </div>
          </div>
        </div>
      </div>
      {showDialog === "deploy" && (
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
      )}
    </>
  );
};

export default memo(ActionBar);
