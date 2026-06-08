import { chromium, FullConfig } from "@playwright/test";

import { HomePage } from "./pages/homePage";
import { LoginPage } from "./pages/loginPage";
import { STORAGE_STATE } from "./playwright.config";

async function globalSetup(_config: FullConfig) {
  const email = process.env.FLOW_DASHBOARD_E2E_USER_EMAIL;
  const password = process.env.FLOW_DASHBOARD_E2E_USER_PASSWORD;
  const baseUrl = process.env.FLOW_DASHBOARD_E2E_BASEURL;

  if (!email || !password || !baseUrl) {
    throw new Error(
      "Missing required env vars: FLOW_DASHBOARD_E2E_USER_EMAIL, FLOW_DASHBOARD_E2E_USER_PASSWORD, FLOW_DASHBOARD_E2E_BASEURL",
    );
  }

  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext();
  const page = await context.newPage();

  try {

    let lastError: unknown;
    for (let attempt = 1; attempt <= 2; attempt++) {
      try {
        await page.goto(baseUrl, {
          waitUntil: "domcontentloaded",
          timeout: 90_000,
        });
        lastError = undefined;
        break;
      } catch (error) {
        lastError = error;
        console.warn(`Global setup navigation attempt ${attempt} failed`);
      }
    }
    if (lastError) throw lastError;

    const loginPage = new LoginPage(page);
    const homePage = new HomePage(page);

    if (!(await loginPage.isLoggedIn())) {
      await loginPage.login(email, password);
      await loginPage.waitForLoggedIn();
    }

    await homePage.waitForLoaded();

    await page.context().storageState({ path: STORAGE_STATE });

    console.log("Global setup completed — authentication state saved");
  } catch (error) {
    console.error("Global setup failed:", error);
    throw error;
  } finally {
    await context.close();
    await browser.close();
  }
}

export default globalSetup;
