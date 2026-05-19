import { expect, test } from "@playwright/test";

import { HomePage } from "../pages/homePage";

test.describe("Smoke", { tag: "@smoke" }, () => {
  test("dashboard home renders with saved auth state", async ({ page }) => {
    const home = new HomePage(page);
    await page.goto("/");
    await home.waitForLoaded();
    await expect(home.brandText).toBeVisible();
    await expect(home.newProjectButton).toBeVisible();
    await expect(home.searchInput).toBeVisible();
  });
});
