import * as path from "path";

import { expect, test } from "@playwright/test";

import { ProjectsPage } from "../pages/projectsPage";

const FIXTURE = path.resolve(__dirname, "fixtures/sample-workflow.json");
const IMPORTED_PROJECT_NAME = "E2E Sample Pipeline (import)";

test.describe("Workflow import", { tag: "@regression" }, () => {
  let projects: ProjectsPage;

  test.beforeEach(async ({ page }) => {
    projects = new ProjectsPage(page);
    await projects.goto();
    await projects.deleteProjectIfExists(IMPORTED_PROJECT_NAME).catch(() => {});
  });

  test.afterEach(async () => {
    await projects.goto();
    await projects.deleteProjectIfExists(IMPORTED_PROJECT_NAME).catch(() => {});
  });

  test("imports a workflow JSON file as a new project", async ({ page }) => {
    await projects.importTrigger.click();
    const workflowItem = page.getByRole("menuitem", { name: /Workflow/ });
    await expect(workflowItem).toBeVisible();

    // The menu item triggers a hidden file input rather than a normal upload.
    const fileChooserPromise = page.waitForEvent("filechooser");
    await workflowItem.click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(FIXTURE);

    await projects.search(IMPORTED_PROJECT_NAME);
    await expect(projects.projectCard(IMPORTED_PROJECT_NAME)).toBeVisible({
      timeout: 30_000,
    });
  });
});
