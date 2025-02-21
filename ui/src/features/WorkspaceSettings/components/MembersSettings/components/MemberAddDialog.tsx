import { memo } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  DialogFooter,
  Input,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { Workspace } from "@flow/types";

type Props = {
  setShowDialog: (show: boolean) => void;
  email: string;
  setEmail: (email: string) => void;
  currentWorkspace?: Workspace;
  onAddMember: (email: string) => void;
  error?: string;
};

const MemberAddDialog: React.FC<Props> = ({
  setShowDialog,
  email,
  setEmail,
  currentWorkspace,
  onAddMember,
  error,
}) => {
  const t = useT();

  return (
    <Dialog open={true} onOpenChange={() => setShowDialog(false)}>
      <DialogContent size="sm">
        <DialogTitle>{t("Add a New User")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection className="flex flex-col">
            <Input
              className="w-full"
              placeholder={t("Enter email")}
              value={email}
              disabled={currentWorkspace?.personal}
              onChange={(e) => setEmail(e.target.value)}
            />
            <p className="text-sm text-red-400">{error}</p>
          </DialogContentSection>
        </DialogContentWrapper>
        <DialogFooter>
          <Button
            onClick={() => onAddMember(email)}
            disabled={!email || currentWorkspace?.personal}>
            {t("Add Member")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default memo(MemberAddDialog);
