// NOTE: if you add more languages, make sure to update
// locales in DateTimePicker component (src/components/DateTimePicker/index.tsx)
export const availableLanguages = ["en", "ja", "es", "fr", "zh"] as const;

export type AvailableLanguage = (typeof availableLanguages)[number];

export const localesWithLabel: { [l in AvailableLanguage]: string } = {
  en: "English",
  ja: "日本語",
  es: "Español",
  fr: "Français",
  zh: "中文",
};
