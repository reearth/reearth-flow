import { ReactNode, useEffect } from "react";
import { I18nextProvider } from "react-i18next";

import { useUser } from "../gql";

import i18n from "./i18n";

export default function Provider({ children }: { children?: ReactNode }) {
  const { useGetMe } = useUser();
  const { me } = useGetMe();
  const selectedLanguage =
    me?.lang && me.lang !== "und" ? me.lang : i18n.language;

  useEffect(() => {
    if (selectedLanguage) {
      i18n.changeLanguage(selectedLanguage);
    }
  }, [selectedLanguage]);
  return <I18nextProvider i18n={i18n}>{children}</I18nextProvider>;
}
