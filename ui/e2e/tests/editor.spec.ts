import { expect, test } from "@playwright/test";

import { EditorPage } from "../pages/editorPage";
import { ProjectsPage, uniqueProjectName } from "../pages/projectsPage";

test.describe.serial("Editor canvas", { tag: "@regression" }, () => {
  let projects: ProjectsPage;
  let editor: EditorPage;
  let projectName: string;

  test.beforeEach(async ({ page }) => {
    projects = new ProjectsPage(page);
    editor = new EditorPage(page);
    projectName = uniqueProjectName("editor");

    await projects.goto();
    await projects.createProject(projectName, "created by e2e editor test");
    await projects.search(projectName);
    await projects.openProject(projectName);
    await editor.waitForLoaded();
  });

  test.afterEach(async () => {
    await projects.goto();
    await projects.deleteProjectIfExists(projectName).catch(() => {});
  });

  test("adds Reader, Transformer and Writer nodes from the palette", async ({
    page,
  }) => {
    const base = await editor.nodeCount();

    const readerName = await editor.addActionNode(
      "reader",
      await editor.canvasPoint(0.28, 0.4),
    );
    const transformerName = await editor.addActionNode(
      "transformer",
      await editor.canvasPoint(0.5, 0.62),
    );
    const writerName = await editor.addActionNode(
      "writer",
      await editor.canvasPoint(0.72, 0.4),
    );

    await expect(editor.nodes).toHaveCount(base + 3);
    await expect(page.locator(".react-flow__node-reader")).toHaveCount(1);
    await expect(page.locator(".react-flow__node-transformer")).toHaveCount(1);
    await expect(page.locator(".react-flow__node-writer")).toHaveCount(1);
    await expect(editor.nodeByName(readerName)).toBeVisible();
    await expect(editor.nodeByName(transformerName)).toBeVisible();
    await expect(editor.nodeByName(writerName)).toBeVisible();
  });

  test("connects two nodes with an edge", async () => {
    const baseEdges = await editor.edgeCount();

    await editor.addActionNode(
      "transformer",
      await editor.canvasPoint(0.32, 0.45),
    );
    await editor.addActionNode(
      "transformer",
      await editor.canvasPoint(0.68, 0.45),
    );
    await expect(editor.nodes).toHaveCount(2);

    await editor.connectNodes(editor.nodes.nth(0), editor.nodes.nth(1));

    await expect(editor.edges).toHaveCount(baseEdges + 1);
  });

  test("configures a node and the value persists on reopen", async () => {
    await editor.addActionNode(
      "transformer",
      await editor.canvasPoint(0.5, 0.5),
    );
    const node = editor.nodes.first();

    const value = `e2e-config-${Date.now()}`;
    await editor.setNodeCustomization(node, value);

    const readBack = await editor.readNodeCustomization(node);
    expect(readBack).toBe(value);
    await editor.closeParamsDialog();
  });

  test("undo and redo restore canvas state", async () => {
    const base = await editor.nodeCount();

    await editor.addActionNode(
      "transformer",
      await editor.canvasPoint(0.5, 0.5),
    );
    await expect(editor.nodes).toHaveCount(base + 1);

    await editor.undo();
    await expect(editor.nodes).toHaveCount(base);

    await editor.redo();
    await expect(editor.nodes).toHaveCount(base + 1);
  });

  test("copies and pastes a node", async () => {
    const name = await editor.addActionNode(
      "transformer",
      await editor.canvasPoint(0.4, 0.4),
    );
    await expect(editor.nodes).toHaveCount(1);

    await editor.copyNode(editor.nodes.first());
    await editor.pasteAtPane(await editor.canvasPoint(0.65, 0.65));

    await expect(editor.nodes).toHaveCount(2);
    await expect(editor.nodeByName(name)).toHaveCount(2);
  });

  test("copies connected nodes including a subworkflow node and preserves the edge", async () => {
    await editor.addActionNode(
      "transformer",
      await editor.canvasPoint(0.3, 0.45),
    );
    await editor.dragToolToCanvas(
      "subworkflow",
      await editor.canvasPoint(0.65, 0.45),
    );
    await expect(editor.nodes).toHaveCount(2);

    await editor.connectNodes(editor.nodes.nth(0), editor.nodes.nth(1));
    await expect(editor.edges).toHaveCount(1);

    await editor.clickPane();
    await editor.selectAll();
    await editor.copySelected();

    await editor.pasteAtPane(await editor.canvasPoint(0.5, 0.75));

    await expect(editor.nodes).toHaveCount(4);
    await expect(editor.edges).toHaveCount(2);
  });

  test("deletes a node and its connected edge", async () => {
    await editor.addActionNode(
      "transformer",
      await editor.canvasPoint(0.32, 0.45),
    );
    await editor.addActionNode(
      "transformer",
      await editor.canvasPoint(0.68, 0.45),
    );
    await editor.connectNodes(editor.nodes.nth(0), editor.nodes.nth(1));
    await expect(editor.edges).toHaveCount(1);

    await editor.selectNode(editor.nodes.nth(0));
    await editor.deleteSelected();

    await expect(editor.nodes).toHaveCount(1);
    await expect(editor.edges).toHaveCount(0);
  });
});
