import { defineConfig } from "allure";

export default defineConfig({
  name: "Re:Earth Flow E2E",
  output: "allure-report",
  // Cross-run trends live in this single JSONL file (Allure 3 replaced the
  // Allure 2 `history/` folder). The CI workflow restores it from GCS before
  // generating and re-uploads it afterwards; `allure generate` reads it for
  // trend/retry charts and appends the current run. Kept in sync with the
  // 50-run cap on the dashboard index.json.
  historyPath: "history.jsonl",
  historyLimit: 50,
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
