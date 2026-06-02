import * as path from "path";

import { expect, test } from "@playwright/test";

import {
  DeploymentsPage,
  uniqueDeploymentDescription,
} from "../pages/deploymentsPage";
import { EditorPage } from "../pages/editorPage";
import { ProjectsPage, uniqueProjectName } from "../pages/projectsPage";

const WORKFLOW_FIXTURE = path.resolve(
  __dirname,
  "fixtures/sample-workflow.json",
);

test.describe("Deployment lifecycle", { tag: "@regression" }, () => {
  test.describe.configure({ timeout: 150_000 });

  let projects: ProjectsPage;
  let editor: EditorPage;
  let deployments: DeploymentsPage;
  let projectName: string;
  let deploymentDescription: string;
  let deploymentDescriptions: string[];

  const newDeploymentDescription = (label?: string) => {
    const description = uniqueDeploymentDescription(label);
    deploymentDescriptions.push(description);
    return description;
  };

  test.beforeEach(async ({ page }) => {
    projects = new ProjectsPage(page);
    editor = new EditorPage(page);
    deployments = new DeploymentsPage(page);
    projectName = uniqueProjectName("deployment");
    deploymentDescriptions = [];
    deploymentDescription = newDeploymentDescription();

    await projects.goto();
    await projects.createProject(projectName, "created by e2e deployment test");
    await projects.search(projectName);
    await projects.openProject(projectName);
    await editor.waitForLoaded();
  });

  test.afterEach(async () => {
    await deployments.goto();
    for (const description of deploymentDescriptions) {
      await deployments.deleteDeploymentIfExists(description).catch(() => { });
    }
    await projects.goto();
    await projects.deleteProjectIfExists(projectName).catch(() => { });
  });

  test("deploys a workflow and it appears in the Deployments table with correct metadata", async () => {
    await editor.addActionNode(
      "transformer",
      await editor.canvasPoint(0.5, 0.5),
    );

    await editor.deploy(deploymentDescription);

    await deployments.goto();
    await deployments.search(deploymentDescription);

    const row = deployments.deploymentRow(deploymentDescription);
    await expect(row).toHaveCount(1);
    await expect(row).toContainText(projectName);
    await expect(deployments.versionCell(deploymentDescription)).toHaveText(
      "v1",
    );
    await expect(deployments.updatedAtCell(deploymentDescription)).toHaveText(
      /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$/,
    );
  });

  test("re-deploying the same project updates the deployment and bumps the version", async () => {
    await editor.addActionNode(
      "transformer",
      await editor.canvasPoint(0.5, 0.5),
    );
    await editor.deploy(deploymentDescription);

    const updatedDescription = newDeploymentDescription("update");
    await editor.updateDeployment(updatedDescription);

    await deployments.goto();
    await deployments.search(updatedDescription);

    const row = deployments.deploymentRow(updatedDescription);
    await expect(row).toHaveCount(1);
    await expect(row).toContainText(projectName);
    await expect(deployments.versionCell(updatedDescription)).toHaveText("v2");
    await deployments.search(deploymentDescription);
    await expect(deployments.emptyState).toBeVisible();
  });

  test("shows deployment details and navigates back to the list", async ({
    page,
  }) => {
    await editor.addActionNode(
      "transformer",
      await editor.canvasPoint(0.5, 0.5),
    );
    await editor.deploy(deploymentDescription);

    await deployments.goto();
    await deployments.search(deploymentDescription);
    await deployments.openDetails(deploymentDescription);

    await expect(page).toHaveURL(/\/workspaces\/[^/]+\/deployments\/[^/]+$/);
    await expect(
      page.getByText(deploymentDescription, { exact: true }),
    ).toBeVisible();
    await expect(page.getByText(projectName, { exact: true })).toBeVisible();
    await expect(page.getByText("v1", { exact: true })).toBeVisible();
    await expect(deployments.detailsRunButton).toBeVisible();
    await expect(deployments.detailsEditButton).toBeVisible();
    await expect(deployments.detailsDeleteButton).toBeVisible();

    await page.goBack();
    await deployments.waitForLoaded();
  });

  test("edits the deployment description from the details view", async ({
    page,
  }) => {
    await editor.addActionNode(
      "transformer",
      await editor.canvasPoint(0.5, 0.5),
    );
    await editor.deploy(deploymentDescription);

    await deployments.goto();
    await deployments.search(deploymentDescription);
    await deployments.openDetails(deploymentDescription);

    await deployments.detailsEditButton.click();
    await expect(deployments.editDialog).toBeVisible();
    await expect(deployments.editDescriptionInput).toHaveValue(
      deploymentDescription,
    );
    await expect(deployments.editSubmitButton).toBeDisabled();

    const updatedDescription = newDeploymentDescription("edit");
    await deployments.editDescriptionInput.fill(updatedDescription);
    await expect(deployments.editSubmitButton).toBeEnabled();
    await deployments.editSubmitButton.click();
    await expect(deployments.deploymentUpdatedToast).toBeVisible({
      timeout: 30_000,
    });

    // Verify the rename through the table.
    await page.goBack();
    await deployments.waitForLoaded();
    await deployments.search(updatedDescription);
    await expect(deployments.deploymentRow(updatedDescription)).toHaveCount(1);
    await deployments.search(deploymentDescription);
    await expect(deployments.emptyState).toBeVisible();
  });

  test("deletes a deployment from the details view", async () => {
    await editor.addActionNode(
      "transformer",
      await editor.canvasPoint(0.5, 0.5),
    );
    await editor.deploy(deploymentDescription);

    await deployments.goto();
    await deployments.search(deploymentDescription);
    await deployments.openDetails(deploymentDescription);

    await deployments.deleteFromDetails();

    await deployments.waitForLoaded();
    await deployments.search(deploymentDescription);
    await expect(deployments.emptyState).toBeVisible();
  });

  test("disables the deploy submit button until a description is entered", async () => {
    await editor.openDeployPopover();
    await expect(editor.deploySubmitButton).toBeDisabled();

    await editor.deployDescriptionInput.fill(deploymentDescription);
    await expect(editor.deploySubmitButton).toBeEnabled();

    await editor.deployDescriptionInput.fill("   ");
    await expect(editor.deploySubmitButton).toBeDisabled();
  });

  test("runs a deployment and a job is created for it", async ({ page }) => {
    await editor.addActionNode(
      "transformer",
      await editor.canvasPoint(0.5, 0.5),
    );
    await editor.deploy(deploymentDescription);

    await deployments.goto();
    await deployments.search(deploymentDescription);
    await deployments.openDetails(deploymentDescription);

    await deployments.runFromDetails();

    await expect(page).toHaveURL(/\/workspaces\/[^/]+\/jobs\/[^/]+$/, {
      timeout: 15_000,
    });
    await expect(page.getByText("Job Details", { exact: true })).toBeVisible();
    await expect(
      page.getByText(deploymentDescription, { exact: true }),
    ).toBeVisible({ timeout: 15_000 });
    await expect(
      page.getByText(/^(queued|running|completed|failed|cancelled)$/).first(),
    ).toBeVisible({ timeout: 15_000 });

    const cancelJob = page.getByRole("button", { name: "Cancel Job" });
    await cancelJob
      .click({ timeout: 10_000 })
      .then(() =>
        expect(page.getByText("Job Cancelled", { exact: true })).toBeVisible({
          timeout: 15_000,
        }),
      )
      .catch(() => { });
  });
});

test.describe("Deployment from file upload", { tag: "@regression" }, () => {
  let deployments: DeploymentsPage;
  let deploymentDescription: string;

  test.beforeEach(async ({ page }) => {
    deployments = new DeploymentsPage(page);
    deploymentDescription = uniqueDeploymentDescription("file");
    await deployments.goto();
  });

  test.afterEach(async () => {
    await deployments.goto();
    await deployments
      .deleteDeploymentIfExists(deploymentDescription)
      .catch(() => { });
  });

  test("creates a deployment from an uploaded workflow file", async () => {
    await deployments.createFromFile(WORKFLOW_FIXTURE, deploymentDescription);

    await deployments.search(deploymentDescription);

    const row = deployments.deploymentRow(deploymentDescription);
    await expect(row).toHaveCount(1);

    await expect(deployments.versionCell(deploymentDescription)).toHaveText(
      "v0",
    );
  });
});
