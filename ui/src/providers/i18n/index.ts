import { useTranslation } from "react-i18next";

export { default as I18nProvider } from "./Provider";
export * from "./locales";

export const useT = () => useTranslation().t;
export const useLang = () => useTranslation().i18n.language;
