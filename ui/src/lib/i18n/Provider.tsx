import type { ReactNode } from "react";
import { useEffect } from "react";
import { I18nextProvider } from "react-i18next";

import { useUser } from "../gql";

import i18n from "./i18n";
import { availableLanguages } from "./locales";

export default function Provider({ children }: { children?: ReactNode }) {
  const { useGetMe } = useUser();
  const { me } = useGetMe();
  const accountLanguage = (availableLanguages as readonly string[]).includes(
    me?.lang ?? "",
  )
    ? me?.lang
    : undefined;
  const selectedLanguage = accountLanguage ?? i18n.resolvedLanguage;

  useEffect(() => {
    if (selectedLanguage) {
      i18n.changeLanguage(selectedLanguage);
    }
  }, [selectedLanguage]);
  return <I18nextProvider i18n={i18n}>{children}</I18nextProvider>;
}
