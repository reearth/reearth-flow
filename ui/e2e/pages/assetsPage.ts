import { expect, Locator, Page } from "@playwright/test";

import { HomePage } from "./homePage";

export type UploadedAsset = { id: string; name: string; url: string };

export class AssetsPage {
  readonly heading: Locator;
  readonly uploadButton: Locator;
  readonly fileInput: Locator;
  readonly searchInput: Locator;
  readonly assetCreatedToast: Locator;

  constructor(private page: Page) {
    this.heading = page.locator("p.text-lg", { hasText: "Workspace Assets" });
    this.uploadButton = page.getByRole("button", { name: "Upload" });
    this.fileInput = page.locator('input[type="file"]');
    this.searchInput = page.getByPlaceholder("Search...");
    this.assetCreatedToast = page.getByText("Asset Created", { exact: true });
  }

  async goto() {
    const home = new HomePage(this.page);
    await this.page.goto("/");
    await home.waitForLoaded();
    await home.navigateTo("Workspace Assets");
    await this.waitForLoaded();
  }

  async waitForLoaded() {
    await this.heading.waitFor({ state: "visible", timeout: 30_000 });
    await this.uploadButton.waitFor({ state: "visible", timeout: 30_000 });
  }

  async upload(filePath: string): Promise<UploadedAsset> {
    const [response] = await Promise.all([
      this.page.waitForResponse(
        async (r) => {
          if (!r.url().includes("graphql")) return false;
          try {
            const json = await r.json();
            return Boolean(json?.data?.createAsset?.asset?.url);
          } catch {
            return false;
          }
        },
        { timeout: 180_000 },
      ),
      this.fileInput.setInputFiles(filePath),
    ]);

    const asset = (await response.json()).data.createAsset.asset;
    await expect(this.assetCreatedToast)
      .toBeVisible({ timeout: 30_000 })
      .catch(() => { });
    return { id: asset.id, name: asset.name, url: asset.url };
  }

  assetCard(name: string): Locator {
    return this.page
      .locator("div.group.cursor-pointer")
      .filter({ has: this.page.getByText(name, { exact: true }) });
  }

  async deleteAssetIfExists(name: string) {
    for (let guard = 0; guard < 25; guard++) {
      await this.searchInput.fill(name);
      const card = this.assetCard(name).first();
      const visible = await card
        .waitFor({ state: "visible", timeout: 8_000 })
        .then(() => true)
        .catch(() => false);
      if (!visible) break;

      await card.hover();
      await card.locator('[data-slot="dropdown-menu-trigger"]').click();
      await this.page.getByRole("menuitem", { name: "Delete Asset" }).click();
      const alert = this.page.getByRole("alertdialog");
      await alert.getByRole("button", { name: "Continue" }).click();
      await alert.waitFor({ state: "hidden" });
    }
    await this.searchInput.fill("");
  }
}
