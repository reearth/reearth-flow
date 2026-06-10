import path from "path";

import { defineConfig, devices } from "@playwright/test";
import * as dotenv from "dotenv";

dotenv.config({ path: path.resolve(__dirname, ".env") });

export const STORAGE_STATE = path.join(__dirname, ".auth/user.json");

export default defineConfig({
  testDir: "./tests",
  globalSetup: process.env.SKIP_STORAGE_STATE
    ? undefined
    : require.resolve("./global-setup"),
  timeout: 80_000,
  expect: { timeout: 15_000 },
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 5 : undefined,
  reporter: process.env.CI
    ? [["html", { open: "never" }], ["list"]]
    : [["html"], ["list"]],
  use: {
    baseURL: process.env.FLOW_DASHBOARD_E2E_BASEURL,
    storageState: process.env.SKIP_STORAGE_STATE ? undefined : STORAGE_STATE,
    actionTimeout: 25_000,
    navigationTimeout: 25_000,
    trace: "on-first-retry",
    screenshot: "only-on-failure",
    video: "retain-on-failure",
    viewport: { width: 1920, height: 1080 },
    locale: "en-US",
  },
  projects: [
    {
      name: "chromium",
      use: {
        ...devices["Desktop Chrome"],
        launchOptions: {
          slowMo: process.env.CI ? 0 : 10,
        },
      },
    },
  ],
});
