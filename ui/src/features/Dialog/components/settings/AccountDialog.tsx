import { useState } from "react";

import { Button, Input, Label } from "@flow/components";
import { useUser } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";

import { ContentHeader, ContentSection } from "..";

type Errors = "failed" | "passwordNotSame" | "passwordFailed" | "emailFailed";

const AccountDialogContent: React.FC = () => {
  const t = useT();
  const { useGetMe, updateMe } = useUser();
  const { me, isLoading } = useGetMe();
  const [name, setName] = useState<string | undefined>(me?.name);
  const [email, setEmail] = useState<string | undefined>(me?.email);
  const [password, setPassword] = useState<string | undefined>();
  const [passwordConfirmation, setPasswordConfirmation] = useState<string | undefined>();
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
      showError === "passwordFailed" ? setShowError("failed") : setShowError("emailFailed");
    }
    setLoading(false);
  };

  return (
    <>
      <ContentHeader title={t("Account settings")} />
      <div className="mx-2">
        <ContentSection
          title={t("Basic information")}
          content={
            <div className="mt-2 flex flex-col gap-6">
              <div>
                <Label htmlFor="user-name">{t("User Name")}</Label>
                <Input
                  id="user-name"
                  placeholder={t("User Name")}
                  disabled={isLoading}
                  value={name}
                  onChange={e => setName(e.target.value)}
                />
              </div>
              <div>
                <Label htmlFor="user-email">{t("Email")}</Label>
                <Input
                  id="user-email"
                  placeholder={t("User Name")}
                  disabled={isLoading}
                  value={email}
                  onChange={e => setEmail(e.target.value)}
                />
              </div>
              <div>
                <Label htmlFor="password">{t("Password")}</Label>
                <Input
                  id="password"
                  placeholder={t("Password")}
                  disabled={isLoading}
                  value={password}
                  type="password"
                  onChange={e => setPassword(e.target.value)}
                />
              </div>
              <div>
                <Label htmlFor="confirm-password">{t("Confirm Password")}</Label>
                <Input
                  id="confirm-password"
                  placeholder={t("Confirm Password")}
                  disabled={isLoading}
                  value={passwordConfirmation}
                  type="password"
                  onChange={e => setPasswordConfirmation(e.target.value)}
                />
              </div>
              <div>
                <Button
                  className="self-end"
                  disabled={isLoading || loading || !name || !email}
                  onClick={handleUpdateMe}>
                  {t("Save")}
                </Button>
              </div>
              <div className={`text-xs text-destructive ${showError ? "opacity-70" : "opacity-0"}`}>
                {showError === "failed" && t("Failed to update the user")}
                {showError === "passwordNotSame" &&
                  t("Password and Confirm password are not the same")}
                {showError === "passwordFailed" && t("Failed to update the password")}
                {showError === "emailFailed" && t("Failed to update email and name")}
              </div>
            </div>
          }
        />
      </div>
    </>
  );
};

export { AccountDialogContent };
