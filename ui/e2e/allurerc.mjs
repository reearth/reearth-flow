import { defineConfig } from "allure";

export default defineConfig({
  name: "Re:Earth Flow E2E",
  output: "allure-report",
  plugins: {
    awesome: {
      options: {
        // Emit a single self-contained HTML file so the CI artifact opens
        // directly in a browser without needing a static file server.
        singleFile: true,
        reportLanguage: "en",
      },
    },
  },
});
