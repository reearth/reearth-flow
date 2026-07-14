import { defineConfig } from "allure";

export default defineConfig({
  name: "Re:Earth Flow E2E",
  output: "allure-report",
  plugins: {
    awesome: {
      options: {
        // Multi-file output is required: reports are served from GCS via
        // https://allure.test.reearth.dev, and Cloud Run rejects responses
        // over 32MB — a single-file report with inlined attachments (session
        // videos, screenshots) exceeds that. To view a downloaded CI artifact
        // locally, run `npm run allure:open` (from ui/e2e).
        singleFile: false,
        reportLanguage: "en",
      },
    },
  },
});
