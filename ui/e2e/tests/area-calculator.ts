import { expect, test, type BrowserContext, type Page } from "@playwright/test";

import { EditorPage } from "../pages/editorPage";
import { ProjectsPage, uniqueProjectName } from "../pages/projectsPage";
import { STORAGE_STATE } from "../playwright.config";


const PARKS = {
  type: "FeatureCollection",
  features: [
    {
      type: "Feature",
      properties: { name: "Imperial Palace East Gardens" },
      geometry: {
        type: "Polygon",
        coordinates: [
          [
            [139.7528, 35.6838],
            [139.7628, 35.6838],
            [139.7628, 35.6919],
            [139.7528, 35.6919],
            [139.7528, 35.6838],
          ],
        ],
      },
    },
    {
      type: "Feature",
      properties: { name: "Yoyogi Park" },
      geometry: {
        type: "Polygon",
        coordinates: [
          [
            [139.6929, 35.6664],
            [139.701, 35.6664],
            [139.701, 35.6751],
            [139.6929, 35.6751],
            [139.6929, 35.6664],
          ],
        ],
      },
    },
    {
      type: "Feature",
      properties: { name: "Ueno Park" },
      geometry: {
        type: "Polygon",
        coordinates: [
          [
            [139.7702, 35.711],
            [139.7774, 35.711],
            [139.7774, 35.7196],
            [139.7702, 35.7196],
            [139.7702, 35.711],
          ],
        ],
      },
    },
  ],
};

test.describe.serial("Area calculator (UI-built) debug run", () => {
  test.describe.configure({ timeout: 420_000 });

  const projectName = uniqueProjectName("area-calc-debug");

  let context: BrowserContext;
  let page: Page;
  let projects: ProjectsPage;
  let editor: EditorPage;

  test.beforeAll(async ({ browser }) => {
    context = await browser.newContext({
      storageState: STORAGE_STATE,
      baseURL: process.env.FLOW_DASHBOARD_E2E_BASEURL,
      viewport: { width: 1920, height: 1080 },
      locale: "en-US",
    });
    page = await context.newPage();
    projects = new ProjectsPage(page);
    editor = new EditorPage(page);
  });

  test.afterAll(async () => {
    if (!context) return;
    try {
      await projects.goto();
      await projects.deleteProjectIfExists(projectName).catch(() => { });
    } finally {
      await context.close();
    }
  });

  test("creates a new project and opens the editor", async () => {
    await projects.goto();
    await projects.createProject(projectName, "created by e2e debug run test");
    await projects.search(projectName);
    await projects.openProject(projectName);
    await editor.waitForLoaded();

    expect(await editor.isInEditor()).toBe(true);
  });

  test("builds GeoJsonReader → AreaCalculator → GeoJsonWriter and configures each node", async () => {
    await editor.addSpecificActionNode(
      "reader",
      "GeoJsonReader",
      await editor.canvasPoint(0.25, 0.4),
    );
    await editor.addSpecificActionNode(
      "transformer",
      "AreaCalculator",
      await editor.canvasPoint(0.5, 0.4),
    );
    await editor.addSpecificActionNode(
      "writer",
      "GeoJsonWriter",
      await editor.canvasPoint(0.75, 0.4),
    );
    await expect(editor.nodes).toHaveCount(3);

    const readerNode = editor.nodeByName("GeoJsonReader");
    const areaNode = editor.nodeByName("AreaCalculator");
    const writerNode = editor.nodeByName("GeoJsonWriter");

    await editor.connectNodes(readerNode, areaNode);
    await editor.connectNodes(areaNode, writerNode);
    await expect(editor.edges).toHaveCount(2);

    await editor.openNodeParamsForm(readerNode);
    await editor.setParamViaValueEditor(
      "Inline Content",
      JSON.stringify(PARKS),
    );
    await editor.submitParams();

    await editor.openNodeParamsForm(areaNode);
    // EPSG:4326 degrees → ~m² near Tokyo (35.7°N): 1 deg² ≈ 1.0029e10 m².
    await editor.setParamText("root_multiplier", "10029000000");
    await editor.setParamText("root_outputAttribute", "areaSqm");
    await editor.submitParams();

    await editor.openNodeParamsForm(writerNode);
    await editor.setParamText(
      "root_output",
      'file::join_path(env.get("workerArtifactPath"), "park-areas.geojson")',
    );
    await editor.submitParams();
  });

  test("starts a debug run and it finishes successfully", async () => {
    await editor.startDebugRun();
    await editor.waitForDebugRunToComplete();
  });

  test("shows engine logs for each node that ran", async () => {
    await expect(
      editor.debugPanel.getByText(/GeoJsonReader - Running/).first(),
    ).toBeVisible({ timeout: 30_000 });
    await expect(
      editor.debugPanel.getByText(/AreaCalculator - Running/).first(),
    ).toBeVisible();
    await expect(
      editor.debugPanel.getByText(/GeoJsonWriter - Running/).first(),
    ).toBeVisible();
    await expect(
      editor.debugPanel.getByText("Workflow finished successfully.").first(),
    ).toBeVisible({ timeout: 30_000 });
  });

  test("lists the one park-areas.geojson output artifact", async () => {
    await expect(editor.debugOutputDataButton).toBeVisible({
      timeout: 60_000,
    });
    await expect(editor.debugOutputDataButton).toContainText("Output data (1)");

    await editor.debugOutputDataButton.click();
    await expect(
      page.getByRole("menuitem", { name: /park-areas\.geojson/ }),
    ).toBeVisible();
    await page.keyboard.press("Escape");
  });
});
