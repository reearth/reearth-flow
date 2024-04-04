import i18n from "i18next";
// import LanguageDetector from "i18next-browser-languagedetector";
import { initReactI18next } from "react-i18next";

import { AvailableLanguage } from "./locales";
import en from "./locales/en.json";
import es from "./locales/es.json";
import fr from "./locales/fr.json";
import ja from "./locales/ja.json";
import zh from "./locales/zh.json";

const resources: { [l in AvailableLanguage]: { translation: any } } = {
  en: {
    translation: en,
  },
  ja: {
    translation: ja,
  },
  es: {
    translation: es,
  },
  fr: {
    translation: fr,
  },
  zh: {
    translation: zh,
  },
};

export const availableLanguages = Object.keys(resources);

i18n
  //   .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    fallbackLng: "en",
    nsSeparator: false,
    keySeparator: false,
    returnEmptyString: false,
    interpolation: {
      escapeValue: false, // not needed for react as it escapes by default
    },
  });

export default i18n;
