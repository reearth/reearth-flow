import { expect, Locator, Page } from "@playwright/test";

import { HomePage } from "./homePage";

const E2E_DEPLOYMENT_PREFIX = "e2e-deployment-";

export function uniqueDeploymentDescription(label = "deploy"): string {
  return `${E2E_DEPLOYMENT_PREFIX}${label}-${Date.now()}-${Math.floor(
    Math.random() * 1000,
  )}`;
}

export class DeploymentsPage {
  readonly heading: Locator;
  readonly newDeploymentButton: Locator;
  readonly searchInput: Locator;
  readonly emptyState: Locator;
  readonly detailsTitle: Locator;
  readonly detailsRunButton: Locator;
  readonly detailsEditButton: Locator;
  readonly detailsDeleteButton: Locator;
  readonly editDialog: Locator;
  readonly editDescriptionInput: Locator;
  readonly editSubmitButton: Locator;
  readonly runDialog: Locator;
  readonly runConfirmButton: Locator;
  readonly addDialog: Locator;
  readonly addDialogFileInput: Locator;
  readonly addDialogDescriptionInput: Locator;
  readonly addDialogSubmitButton: Locator;
  readonly deploymentCreatedToast: Locator;
  readonly deploymentUpdatedToast: Locator;
  readonly deploymentDeletedToast: Locator;
  readonly deploymentExecutedToast: Locator;

  constructor(private page: Page) {
    this.heading = page.locator("p.text-lg", { hasText: "Deployments" });
    this.newDeploymentButton = page.getByRole("button", {
      name: "New Deployment",
    });
    this.searchInput = page.getByPlaceholder("Search...");
    this.emptyState = page.getByText("No Deployments");

    this.detailsTitle = page.getByText("Deployment Details", { exact: true });
    this.detailsRunButton = page.getByRole("button", {
      name: "Run",
      exact: true,
    });
    this.detailsEditButton = page.getByRole("button", {
      name: "Edit Deployment",
    });
    this.detailsDeleteButton = page.getByRole("button", {
      name: "Delete",
      exact: true,
    });

    this.editDialog = page
      .getByRole("dialog")
      .filter({ hasText: "Edit Deployment" });
    this.editDescriptionInput = this.editDialog.getByPlaceholder(
      "Give your deployment a meaningful description...",
    );
    this.editSubmitButton = this.editDialog.getByRole("button", {
      name: "Update Deployment",
    });

    this.runDialog = page
      .getByRole("dialog")
      .filter({ hasText: "Run Deployment" });
    this.runConfirmButton = this.runDialog.getByRole("button", {
      name: "Run",
      exact: true,
    });

    this.addDialog = page
      .getByRole("dialog")
      .filter({ hasText: "Create a deployment from file" });
    this.addDialogFileInput = this.addDialog.locator('input[type="file"]');
    this.addDialogDescriptionInput = this.addDialog.getByPlaceholder(
      "Give your deployment a meaningful description...",
    );
    this.addDialogSubmitButton = this.addDialog.getByRole("button", {
      name: "Deploy",
      exact: true,
    });

    this.deploymentCreatedToast = page.getByText("Deployment Created", {
      exact: true,
    });
    this.deploymentUpdatedToast = page.getByText("Deployment Updated", {
      exact: true,
    });
    this.deploymentDeletedToast = page.getByText("Successful Deletion", {
      exact: true,
    });
    this.deploymentExecutedToast = page.getByText("Deployment Executed", {
      exact: true,
    });
  }

  async goto() {
    const home = new HomePage(this.page);
    await this.page.goto("/");
    await home.waitForLoaded();
    await home.navigateTo("Deployments");
    await this.waitForLoaded();
  }

  async waitForLoaded() {
    await this.heading.waitFor({ state: "visible", timeout: 30_000 });
    await this.newDeploymentButton.waitFor({ state: "visible" });
  }

  deploymentRow(description: string): Locator {
    return this.page
      .locator("tbody tr")
      .filter({ has: this.page.getByText(description, { exact: true }) });
  }

  // Table column order: Description, Project Name, Version, Updated At,
  // Quick Actions.
  versionCell(description: string): Locator {
    return this.deploymentRow(description).first().locator("td").nth(2);
  }

  updatedAtCell(description: string): Locator {
    return this.deploymentRow(description).first().locator("td").nth(3);
  }

  async search(term: string) {
    await this.searchInput.fill(term);
  }

  async clearSearch() {
    await this.searchInput.fill("");
  }

  async openDetails(description: string) {
    const row = this.deploymentRow(description).first();
    await row.getByText(description, { exact: true }).click();
    await expect(this.detailsTitle).toBeVisible();
  }

  async editFromDetails(newDescription: string) {
    await this.detailsEditButton.click();
    await expect(this.editDialog).toBeVisible();
    await this.editDescriptionInput.fill(newDescription);
    await this.editSubmitButton.click();
    await expect(this.deploymentUpdatedToast).toBeVisible({ timeout: 30_000 });
    await expect(this.editDialog).toBeHidden();
  }

  async deleteFromDetails() {
    await this.detailsDeleteButton.click();
    const alert = this.page.getByRole("alertdialog");
    await alert.getByRole("button", { name: "Continue" }).click();
    await expect(this.deploymentDeletedToast).toBeVisible({ timeout: 30_000 });
    await alert.waitFor({ state: "hidden" });
  }

  async runFromDetails() {
    await this.detailsRunButton.click();
    await expect(this.runDialog).toBeVisible();
    await this.runConfirmButton.click();
    await expect(this.deploymentExecutedToast).toBeVisible({
      timeout: 30_000,
    });
  }

  async createFromFile(workflowFile: string, description: string) {
    await this.newDeploymentButton.click();
    await expect(this.addDialog).toBeVisible();
    await expect(this.addDialogSubmitButton).toBeDisabled();
    await this.addDialogFileInput.setInputFiles(workflowFile);
    await this.addDialogDescriptionInput.fill(description);
    await expect(this.addDialogSubmitButton).toBeEnabled();
    await this.addDialogSubmitButton.click();
    await expect(this.deploymentCreatedToast).toBeVisible({ timeout: 30_000 });
    await expect(this.addDialog).toBeHidden();
  }

  private async waitForListSettled(description: string) {
    await expect(
      this.deploymentRow(description).first().or(this.emptyState),
    ).toBeVisible({ timeout: 15_000 });
  }

  async deleteDeployment(description: string) {
    const row = this.deploymentRow(description).first();
    await row.waitFor({ state: "visible" });
    await row.getByRole("button").last().click();
    const alert = this.page.getByRole("alertdialog");
    await alert.getByRole("button", { name: "Continue" }).click();
    await alert.waitFor({ state: "hidden" });
    await expect(this.deploymentRow(description)).toHaveCount(0);
  }

  async deleteDeploymentIfExists(description: string) {
    for (let guard = 0; guard < 25; guard++) {
      await this.clearSearch();
      await this.search(description);
      await this.waitForListSettled(description);
      if ((await this.deploymentRow(description).count()) === 0) break;
      await this.deleteDeployment(description);
    }
    await this.clearSearch();
  }
}
