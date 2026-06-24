import { expect, test, type Page } from "@playwright/test";

import {
  type EditorSession,
  newEditorSession,
  teardownSession,
} from "../fixtures/session";
import { EditorPage } from "../pages/editorPage";
import { ProjectsPage, uniqueProjectName } from "../pages/projectsPage";

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

test.describe.serial(
  "Area calculator (UI-built) debug run",
  { tag: "@pipeline" },
  () => {
    test.describe.configure({ timeout: 420_000 });

    const projectName = uniqueProjectName("area-calc-debug");

    let session: EditorSession;
    let page: Page;
    let projects: ProjectsPage;
    let editor: EditorPage;

    test.beforeAll(async ({ browser }) => {
      session = await newEditorSession(browser);
      ({ page, projects, editor } = session);
    });

    test.afterAll(async () => {
      await teardownSession(session, { projectName });
    });

    test("creates a new project and opens the editor", async () => {
      await projects.goto();
      await projects.createProject(
        projectName,
        "created by e2e debug run test",
      );
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
      await editor.setParamLiteralString(
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
      await editor.setParamCodeString("output", "park-areas.geojson");
      await editor.submitParams();
    });

    test("starts a debug run and it finishes successfully", async () => {
      await editor.startDebugRun();
      await editor.waitForDebugRunToComplete();
    });

    test("shows the workflow run logs", async () => {
      await expect(
        editor.debugPanel.getByText(/Workflow - Started/).first(),
      ).toBeVisible({ timeout: 30_000 });
      await expect(
        editor.debugPanel.getByText("Workflow finished successfully.").first(),
      ).toBeVisible();
    });

    test("lists the one park-areas.geojson output artifact", async () => {
      await expect(editor.debugOutputDataButton).toBeVisible({
        timeout: 60_000,
      });
      await expect(editor.debugOutputDataButton).toContainText(
        "Output data (1)",
      );

      await editor.debugOutputDataButton.click();
      await expect(
        page.getByRole("menuitem", { name: /park-areas\.geojson/ }),
      ).toBeVisible();
      await page.keyboard.press("Escape");
    });
  },
);
