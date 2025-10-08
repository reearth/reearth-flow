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
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
// import { ThemeToggle } from "@flow/features/ThemeToggle";
import { ThemeToggle } from "@flow/features/ThemeToggle";
import { useUser } from "@flow/lib/gql";
import { AvailableLanguage, localesWithLabel, useT } from "@flow/lib/i18n";
import i18n from "@flow/lib/i18n/i18n";

type Errors =
  | "failed"
  | "passwordNotSame"
  | "passwordFailed"
  | "langUpdateFailed";

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
  // For some users me.lang maybe lang: "und". Therefore, we can default to i18n.language.
  const language = me?.lang && me.lang !== "und" ? me?.lang : i18n.language;
  const [selectedLang, setSelectedLang] = useState<string>(language);

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
        setLoading(false);
        return;
      }
    }

    if (selectedLang) {
      const input = { name, lang: selectedLang };
      const { me: user } = await updateMe(input);
      if (!user) {
        setShowError("langUpdateFailed");
        setLoading(false);
        return;
      }
    }

    const input = { name, email };
    const { me: user } = await updateMe(input);
    if (!user) {
      setShowError("failed");
      setLoading(false);
      return;
    }
    setLoading(false);
  };
  const handleLanguageChange = (lang: string) => {
    setSelectedLang(lang);
  };
  const currentLanguageLabel =
    localesWithLabel[i18n.language as AvailableLanguage] ||
    t("Select Language");

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
                placeholder={t("Email")}
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
          {/* <DialogContentSection className="flex-1">
            <Label htmlFor="theme">{t("Theme")}</Label>
            <ThemeToggle />
          </DialogContentSection> */}
          <DialogContentSection className="flex-1">
            <Label htmlFor="language-selector">{t("Select Language")}</Label>
            <Select onValueChange={handleLanguageChange}>
              <SelectTrigger>
                <SelectValue placeholder={currentLanguageLabel} />
              </SelectTrigger>
              <SelectContent>
                {Object.entries(localesWithLabel).map(([value, label]) => (
                  <SelectItem key={value} value={value}>
                    {label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </DialogContentSection>
          <DialogContentSection className="flex-1">
            <Label htmlFor="theme-selector">{t("Select Theme")}</Label>
            <ThemeToggle />
          </DialogContentSection>
        </DialogContentWrapper>
        <div
          className={`text-xs text-destructive ${showError ? "opacity-70" : "opacity-0"}`}>
          {showError === "failed" && t("Failed to update the user")}
          {showError === "passwordNotSame" &&
            t("Password and Confirm password are not the same")}
          {showError === "passwordFailed" && t("Failed to update the password")}
        </div>
        <DialogFooter>
          <Button
            className="self-end"
            disabled={isLoading || loading || !name || !email}
            onClick={handleUpdateMe}>
            {t("Save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export { AccountUpdateDialog };
