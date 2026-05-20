import { expect, Locator, Page } from "@playwright/test";

export const E2E_PROJECT_PREFIX = "e2e-";

export function uniqueProjectName(label = "project"): string {
  return `${E2E_PROJECT_PREFIX}${label}-${Date.now()}-${Math.floor(
    Math.random() * 1000,
  )}`;
}

export class ProjectsPage {
  readonly heading: Locator;
  readonly newProjectButton: Locator;
  readonly importTrigger: Locator;
  readonly searchInput: Locator;
  readonly emptyState: Locator;

  constructor(private page: Page) {
    this.heading = page.locator("p.text-lg", { hasText: "Projects" });
    this.newProjectButton = page.getByRole("button", { name: "New Project" });
    this.importTrigger = page.getByRole("button", { name: "Import" });
    this.searchInput = page.getByPlaceholder("Search...");
    this.emptyState = page.getByText("No Projects");
  }

  async goto() {
    await this.page.goto("/");
    await this.waitForLoaded();
  }

  async waitForLoaded() {
    await this.heading.waitFor({ state: "visible", timeout: 30_000 });
    await this.newProjectButton.waitFor({ state: "visible" });
  }

  projectCard(name: string): Locator {
    return this.page
      .locator("div.group.cursor-pointer.rounded-xl")
      .filter({ has: this.page.getByText(name, { exact: true }) });
  }

  async search(term: string) {
    await this.searchInput.fill(term);
  }

  async clearSearch() {
    await this.searchInput.fill("");
  }

  async createProject(name: string, description?: string) {
    await this.newProjectButton.click();
    const dialog = this.page
      .getByRole("dialog")
      .filter({ hasText: "New Project" });
    await dialog.getByPlaceholder("Project name...").fill(name);
    if (description) {
      await dialog.getByPlaceholder("Project description...").fill(description);
    }
    await dialog.getByRole("button", { name: "Create" }).click();
    await dialog.waitFor({ state: "hidden" });
  }

  private async openCardMenu(name: string) {
    const card = this.projectCard(name).first();
    await card.waitFor({ state: "visible" });
    await card.hover();
    await card.locator('button[data-slot="dropdown-menu-trigger"]').click();
  }

  async renameProject(oldName: string, newName: string) {
    await this.openCardMenu(oldName);
    await this.page.getByRole("menuitem", { name: "Edit Details" }).click();
    const dialog = this.page
      .getByRole("dialog")
      .filter({ hasText: "Edit Project" });
    const nameInput = dialog.getByPlaceholder("Your project name goes here...");
    await nameInput.fill(newName);
    await dialog.getByRole("button", { name: "Save" }).click();
    await dialog.waitFor({ state: "hidden" });
  }

  async duplicateProject(name: string): Promise<string> {
    await this.openCardMenu(name);
    await this.page
      .getByRole("menuitem", { name: "Duplicate Project" })
      .click();
    const dialog = this.page
      .getByRole("dialog")
      .filter({ hasText: "Duplicate Project" });
    const nameInput = dialog.getByPlaceholder("Your project name goes here...");
    const duplicateName = await nameInput.inputValue();
    await dialog.getByRole("button", { name: "Duplicate" }).click();
    await dialog.waitFor({ state: "hidden" });
    return duplicateName;
  }

  async deleteProject(name: string) {
    await this.openCardMenu(name);
    await this.page.getByRole("menuitem", { name: "Delete Project" }).click();
    const alert = this.page.getByRole("alertdialog");
    await alert.getByRole("button", { name: "Continue" }).click();
    await alert.waitFor({ state: "hidden" });
  }

  async openProject(name: string) {
    await this.projectCard(name).first().click();
  }

  private async waitForListSettled(name: string) {
    await expect(
      this.projectCard(name).first().or(this.emptyState),
    ).toBeVisible({ timeout: 15_000 });
  }

  async deleteProjectIfExists(name: string) {
    for (let guard = 0; guard < 25; guard++) {
      await this.clearSearch();
      await this.search(name);
      await this.waitForListSettled(name);
      if ((await this.projectCard(name).count()) === 0) break;
      await this.deleteProject(name);
    }
    await this.clearSearch();
  }
}
