import { expect, test } from "@playwright/test";

import { EditorPage } from "../pages/editorPage";
import { ProjectsPage, uniqueProjectName } from "../pages/projectsPage";

test.describe("Project lifecycle", { tag: "@smoke" }, () => {
  let projects: ProjectsPage;
  const createdNames: string[] = [];

  test.beforeEach(async ({ page }) => {
    projects = new ProjectsPage(page);
    await projects.goto();
  });

  test.afterEach(async () => {
    for (const name of createdNames.splice(0)) {
      await projects.deleteProjectIfExists(name).catch(() => {});
    }
  });

  test("create, search, rename, open editor, duplicate, then delete both", async ({
    page,
  }) => {
    const editor = new EditorPage(page);

    const name = uniqueProjectName("lifecycle");
    createdNames.push(name);
    await projects.createProject(name, "created by e2e test");

    await projects.search(name);
    await expect(projects.projectCard(name)).toBeVisible();

    const renamed = uniqueProjectName("renamed");
    createdNames.push(renamed);
    await projects.renameProject(name, renamed);
    await projects.search(renamed);
    await expect(projects.projectCard(renamed)).toBeVisible();
    await expect(projects.projectCard(name)).toHaveCount(0);

    await projects.openProject(renamed);
    await expect(page).toHaveURL(/\/projects\/[^/]+$/);
    await editor.waitForLoaded();
    await expect(editor.canvas).toBeVisible();

    await projects.goto();
    await projects.search(renamed);
    await expect(projects.projectCard(renamed)).toBeVisible();
    const duplicateName = await projects.duplicateProject(renamed);
    createdNames.push(duplicateName);
    await projects.search(duplicateName);
    await expect(projects.projectCard(duplicateName)).toBeVisible();

    await projects.deleteProjectIfExists(renamed);
    await projects.deleteProjectIfExists(duplicateName);

    await projects.search(renamed);
    await expect(projects.projectCard(renamed)).toHaveCount(0);
    await projects.search(duplicateName);
    await expect(projects.projectCard(duplicateName)).toHaveCount(0);
  });
});
