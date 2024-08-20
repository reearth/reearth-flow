import { useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  Label,
} from "@flow/components";
import { useUser } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";

type Errors = "failed" | "passwordNotSame" | "passwordFailed" | "emailFailed";

type Props = {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
};

const AccountUpdateDialog: React.FC<Props> = ({ isOpen, onOpenChange }) => {
  const t = useT();
  const { useGetMe, updateMe } = useUser();
  const { me, isLoading } = useGetMe();
  const [name, setName] = useState<string | undefined>(me?.name);
  const [email, setEmail] = useState<string | undefined>(me?.email);
  const [password, setPassword] = useState<string | undefined>();
  const [passwordConfirmation, setPasswordConfirmation] = useState<
    string | undefined
  >();
  const [showError, setShowError] = useState<Errors | undefined>(undefined);
  const [loading, setLoading] = useState(false);

  const handleUpdateMe = async () => {
    setLoading(true);
    setShowError(undefined);
    if (!name || !email) {
      setLoading(false);
      return;
    }
    if (password != passwordConfirmation) {
      setShowError("passwordNotSame");
      setLoading(false);
      return;
    }

    // Update the password if it's changed
    if (password) {
      const input = { name, password, passwordConfirmation };
      const { me: user } = await updateMe(input);
      if (!user) {
        setShowError("passwordFailed");
      }
    }

    const input = { name, email };
    const { me: user } = await updateMe(input);
    if (!user) {
      showError === "passwordFailed"
        ? setShowError("failed")
        : setShowError("emailFailed");
    }
    setLoading(false);
  };

  return (
    <Dialog open={isOpen} onOpenChange={(o) => onOpenChange(o)}>
      <DialogContent size="md">
        <DialogHeader>
          <DialogTitle>{t("Account settings")}</DialogTitle>
        </DialogHeader>
        <DialogContentWrapper>
          <DialogContentSection className="flex-row">
            <DialogContentSection className="flex-1">
              <Label htmlFor="user-name">{t("User Name")}</Label>
              <Input
                id="user-name"
                placeholder={t("User Name")}
                disabled={isLoading}
                value={name}
                onChange={(e) => setName(e.target.value)}
              />
            </DialogContentSection>
            <DialogContentSection className="flex-1">
              <Label htmlFor="user-email">{t("Email")}</Label>
              <Input
                id="user-email"
                placeholder={t("User Name")}
                disabled={isLoading}
                value={email}
                onChange={(e) => setEmail(e.target.value)}
              />
            </DialogContentSection>
          </DialogContentSection>
          <DialogContentSection className="flex-row">
            <DialogContentSection className="flex-1">
              <Label htmlFor="password">{t("Password")}</Label>
              <Input
                id="password"
                placeholder={t("Password")}
                disabled={isLoading}
                value={password}
                type="password"
                onChange={(e) => setPassword(e.target.value)}
              />
            </DialogContentSection>
            <DialogContentSection className="flex-1">
              <Label htmlFor="confirm-password">{t("Confirm Password")}</Label>
              <Input
                id="confirm-password"
                placeholder={t("Confirm Password")}
                disabled={isLoading}
                value={passwordConfirmation}
                type="password"
                onChange={(e) => setPasswordConfirmation(e.target.value)}
              />
            </DialogContentSection>
          </DialogContentSection>
          <div
            className={`text-xs text-destructive ${showError ? "opacity-70" : "opacity-0"}`}
          >
            {showError === "failed" && t("Failed to update the user")}
            {showError === "passwordNotSame" &&
              t("Password and Confirm password are not the same")}
            {showError === "passwordFailed" &&
              t("Failed to update the password")}
            {showError === "emailFailed" &&
              t("Failed to update email and name")}
          </div>
        </DialogContentWrapper>
        <DialogFooter>
          <Button
            className="self-end"
            disabled={isLoading || loading || !name || !email}
            onClick={handleUpdateMe}
          >
            {t("Save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export { AccountUpdateDialog };
