import { useState } from "react";

import { Dialog, DialogContent, DialogFooter } from "@flow/components";
import { Button } from "@flow/components/buttons/BaseButton";
import { useT } from "@flow/providers";

import { ContentHeader } from "../ContentHeader";
import { ContentSection } from "../ContentSection";

import { FieldWrapper } from "./components";

const AccountDialogContent: React.FC = () => {
  const t = useT();
  const [subDialog, setSubDialog] = useState(false);
  return (
    <>
      <ContentHeader
        title={t("Account settings")}
        // description={t("All settings related to your individual account.")}
      />
      <div className="mx-2">
        <ContentSection
          title={t("Basic information")}
          content={
            <div className="flex flex-col gap-6 mt-2">
              <FieldWrapper>
                <div className="mr-4">
                  <p className="text-md">{t("Username")}</p>
                  <p className="text-xs text-zinc-400">flow-user1234</p>
                </div>
                <Button variant="outline" size="sm" onClick={() => setSubDialog(true)}>
                  {t("Change username")}
                </Button>
              </FieldWrapper>
              <FieldWrapper>
                <div className="mr-4">
                  <p className="mr-4 text-md">{t("Email address")}</p>
                  <p className="text-xs text-zinc-400">flow-user@reearth.io</p>
                </div>
                <Button variant="outline" size="sm">
                  {t("Change email address")}
                </Button>
              </FieldWrapper>
              <FieldWrapper>
                <p className="mr-4 text-md">{t("Password")}</p>
                <Button variant="outline" size="sm">
                  {t("Change password")}
                </Button>
              </FieldWrapper>
            </div>
          }
        />
      </div>
      {/* <DialogFooter>
        <Button type="submit">Save changes</Button>
      </DialogFooter> */}
      <Dialog open={!!subDialog} onOpenChange={() => setSubDialog(!subDialog)}>
        <DialogContent overlayBgClass="bg-black/30">
          <ContentHeader title={t("Update field")} />
          <div className="flex gap-4">
            <p className="text-lg text-zinc-300">Field</p>
            <input className="flex-1 bg-zinc-800" type="text" />
          </div>
          <DialogFooter>
            <Button type="submit" onClick={() => setSubDialog(false)}>
              Save changes
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
};

export { AccountDialogContent };
