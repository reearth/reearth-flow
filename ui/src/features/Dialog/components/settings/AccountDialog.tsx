import { useState } from "react";

import { Button, Input, Label } from "@flow/components";
import { useUser } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";

import { ContentHeader, ContentSection } from "..";

const AccountDialogContent: React.FC = () => {
  const t = useT();
  const { useGetMe, updateMe } = useUser();
  const { me, isLoading } = useGetMe();
  const [userName, setUserName] = useState<string | undefined>(me?.name);
  const [email, setEmail] = useState<string | undefined>(me?.email);
  const [showError, setShowError] = useState<boolean>(false);
  const [loading, setLoading] = useState(false);

  const handleUpdateMe = async () => {
    setLoading(true);
    setShowError(false);
    if (!userName || !email) {
      setLoading(false);
      return;
    }
    const { me } = await updateMe({ name: userName, email });
    setLoading(false);
    if (!me) {
      setShowError(true);
      return;
    }
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
                  value={userName}
                  onChange={e => setUserName(e.target.value)}
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
                <Button
                  className="self-end"
                  disabled={isLoading || loading || !userName || !email}
                  onClick={handleUpdateMe}>
                  {t("Save")}
                </Button>
              </div>
              <div
                className={`self-end text-xs text-destructive ${showError ? "opacity-70" : "opacity-0"}`}>
                {showError && t("Failed to update Account details")}
              </div>
            </div>
          }
        />
      </div>
    </>
  );
};

export { AccountDialogContent };
