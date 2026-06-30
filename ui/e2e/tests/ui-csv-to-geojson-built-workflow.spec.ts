import { expect, test, type Page } from "@playwright/test";

import {
  type EditorSession,
  newEditorSession,
  teardownSession,
} from "../fixtures/session";
import { expectJobSucceeded, jobDetailsArtifact } from "../helpers/job";
import {
  DeploymentsPage,
  uniqueDeploymentDescription,
} from "../pages/deploymentsPage";
import { EditorPage } from "../pages/editorPage";
import { ProjectsPage, uniqueProjectName } from "../pages/projectsPage";

const STATIONS_CSV = [
  "name,line,daily_riders,lon,lat",
  "Shinjuku,JR Yamanote,775000,139.7006,35.6896",
  "Shibuya,JR Yamanote,366000,139.7016,35.6580",
  "Tokyo,JR Yamanote,462000,139.7671,35.6812",
  "Ikebukuro,JR Yamanote,558000,139.7109,35.7295",
  "Shinagawa,JR Yamanote,377000,139.7387,35.6285",
].join("\n");

test.describe.serial(
  "CSV to GeoJSON (UI-built) pipeline",
  { tag: "@pipeline" },
  () => {
    test.describe.configure({ timeout: 360_000 });

    const projectName = uniqueProjectName("csv-geojson");
    const deploymentDescription = uniqueDeploymentDescription("csv-geojson");

    let session: EditorSession;
    let page: Page;
    let projects: ProjectsPage;
    let editor: EditorPage;
    let deployments: DeploymentsPage;
    let geojson: any;

    test.beforeAll(async ({ browser }) => {
      session = await newEditorSession(browser);
      ({ page, projects, editor, deployments } = session);
    });

    test.afterAll(async () => {
      await teardownSession(session, { projectName, deploymentDescription });
    });

    test("creates a new project and opens the editor", async () => {
      await projects.goto();
      await projects.createProject(projectName, "created by e2e ui-built test");
      await projects.search(projectName);
      await projects.openProject(projectName);
      await editor.waitForLoaded();

      expect(await editor.isInEditor()).toBe(true);
    });

    test("builds CsvReader → AttributeManager → GeoJsonWriter and configures each node", async () => {
      await editor.addSpecificActionNode(
        "reader",
        "CsvReader",
        await editor.canvasPoint(0.25, 0.4),
      );
      await editor.addSpecificActionNode(
        "transformer",
        "AttributeManager",
        await editor.canvasPoint(0.5, 0.4),
      );
      await editor.addSpecificActionNode(
        "writer",
        "GeoJsonWriter",
        await editor.canvasPoint(0.75, 0.4),
      );
      await expect(editor.nodes).toHaveCount(3);

      const readerNode = editor.nodeByName("CsvReader");
      const managerNode = editor.nodeByName("AttributeManager");
      const writerNode = editor.nodeByName("GeoJsonWriter");

      await editor.connectNodes(readerNode, managerNode);
      await editor.connectNodes(managerNode, writerNode);
      await expect(editor.edges).toHaveCount(2);

      await editor.openNodeParamsForm(readerNode);
      await editor.setParamSelect(
        "File Format",
        "CSV (Comma-Separated Values)",
      );
      await editor.setParamLiteralString("Inline Content", STATIONS_CSV);
      await editor.setCsvCoordinateGeometry("lon", "lat", 4326);
      await editor.submitParams();

      await editor.openNodeParamsForm(managerNode);
      await editor.addParamArrayItem();
      await editor.setParamText("root_operations_0_attribute", "dailyRiders");
      await editor.setParamSelect("Operation to perform", "create");
      await editor.setParamFlowExpr("Value", 'int(attributes["daily_riders"])');
      await editor.submitParams();
      await editor.openNodeParamsForm(writerNode);
      await editor.setParamCodeString("output", "stations.geojson");
      await editor.submitParams();
    });

    test("deploys the workflow", async () => {
      await editor.deploy(deploymentDescription);
    });

    test("runs the deployment as a job and the job completes", async () => {
      await deployments.goto();
      await deployments.search(deploymentDescription);
      await deployments.openDetails(deploymentDescription);
      await deployments.runFromDetails();

      await expect(page).toHaveURL(/\/workspaces\/[^/]+\/jobs\/[^/]+$/, {
        timeout: 15_000,
      });
      test
        .info()
        .annotations.push({ type: "job-url", description: page.url() });

      await expectJobSucceeded(page);
    });

    test("produces stations.geojson listing all five stations", async () => {
      const outputUrl = jobDetailsArtifact(page, "stations.geojson");
      await expect(outputUrl).toBeVisible({ timeout: 90_000 });
      const artifactUrl = (await outputUrl.textContent())?.trim() ?? "";
      test.info().annotations.push({
        type: "output-url",
        description: artifactUrl,
      });

      const response = await page.request.get(artifactUrl);
      expect(response.ok()).toBeTruthy();
      geojson = await response.json();

      expect(geojson.type).toBe("FeatureCollection");
      expect(geojson.features).toHaveLength(5);
    });

    test("converts Shinjuku's daily-riders text into a number", () => {
      const shinjuku = geojson.features.find(
        (f: any) => f.properties?.name === "Shinjuku",
      );
      expect(shinjuku).toBeTruthy();
      expect(shinjuku.properties.dailyRiders).toBe(775000);
    });

    test("turns Shinjuku's lon/lat columns into a map point", () => {
      const shinjuku = geojson.features.find(
        (f: any) => f.properties?.name === "Shinjuku",
      );
      expect(shinjuku).toBeTruthy();
      expect(shinjuku.geometry.type).toBe("Point");
      expect(shinjuku.geometry.coordinates[0]).toBeCloseTo(139.7006, 3);
      expect(shinjuku.geometry.coordinates[1]).toBeCloseTo(35.6896, 3);
    });
  },
);
