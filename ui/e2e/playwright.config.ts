import path from "path";

import { defineConfig, devices } from "@playwright/test";
import * as dotenv from "dotenv";

dotenv.config({ path: path.resolve(__dirname, ".env") });

export const STORAGE_STATE = path.join(__dirname, ".auth/user.json");

const chromiumUse = {
  ...devices["Desktop Chrome"],
  launchOptions: {
    slowMo: process.env.CI ? 0 : 10,
  },
};

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
  workers: 5,
  reporter: [["list"], ["allure-playwright", { resultsDir: "allure-results" }]],
  use: {
    baseURL: process.env.FLOW_DASHBOARD_E2E_BASEURL,
    storageState: process.env.SKIP_STORAGE_STATE ? undefined : STORAGE_STATE,
    actionTimeout: 25_000,
    navigationTimeout: 25_000,
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
    video: "on",
    viewport: { width: 1920, height: 1080 },
    locale: "en-US",
  },
  projects: [
    // Fast UI tests (@smoke/@regression) — safe to run in parallel.
    {
      name: "fast",
      grepInvert: /@pipeline/,
      fullyParallel: true,
      use: chromiumUse,
    },
    // Pipeline tests (@pipeline) deploy and run real engine jobs. They run at
    // the default 5 workers (steps within a serial spec stay ordered via
    // fullyParallel:false). If a loaded dev engine starts flaking concurrent
    // jobs, fall back to the serial `test:pipeline:serial` script (--workers=1).
    {
      name: "pipeline",
      grep: /@pipeline/,
      fullyParallel: false,
      use: chromiumUse,
    },
  ],
});
