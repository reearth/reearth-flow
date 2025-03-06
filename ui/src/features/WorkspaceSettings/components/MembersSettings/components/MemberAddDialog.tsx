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

type Props = {
  setShowDialog: (show: boolean) => void;
  email: string;
  setEmail: (email: string) => void;
  onAddMember: (email: string) => void;
  error?: string;
};

const MemberAddDialog: React.FC<Props> = ({
  setShowDialog,
  email,
  setEmail,
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
              onChange={(e) => setEmail(e.target.value)}
            />
            <p className="text-sm text-red-400">{error}</p>
          </DialogContentSection>
        </DialogContentWrapper>
        <DialogFooter>
          <Button onClick={() => onAddMember(email)} disabled={!email}>
            {t("Add Member")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default memo(MemberAddDialog);
