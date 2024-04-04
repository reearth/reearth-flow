export default {
  locales: ["en", "ja", "es", "fr", "zh"],
  output: "src/providers/i18n/locales/$LOCALE.json",
  input: ["src/**/*.{ts,tsx}"],
  // allow keys to be phrases having `:`, `.`
  namespaceSeparator: false,
  keySeparator: false,
  createOldCatalogs: false,
};
