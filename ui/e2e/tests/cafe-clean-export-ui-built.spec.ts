import * as path from "path";

import { expect, test, type Locator, type Page } from "@playwright/test";

import {
  type EditorSession,
  newEditorSession,
  teardownSession,
} from "../fixtures/session";
import { expectJobSucceeded, jobDetailsArtifact } from "../helpers/job";
import { AssetsPage } from "../pages/assetsPage";
import {
  DeploymentsPage,
  uniqueDeploymentDescription,
} from "../pages/deploymentsPage";
import { EditorPage } from "../pages/editorPage";
import { ProjectsPage, uniqueProjectName } from "../pages/projectsPage";

const CAFES_CSV = path.resolve(
  __dirname,
  "../assets/auckland_cafe_restaurants.csv",
);

const OPEN_PORT = "open_cafes_and_restaurants";
const HIGH_RATED_ATTR = "high_rated";
const GEOJSON_OUTPUT = "clean_data.geojson";
const CSV_OUTPUT = "clean_data.csv";

const TOTAL_ROWS = 80;
const OPEN_ROWS = 64;
const HIGH_RATED_ROWS = 37;
const HIGH_RATED_BUT_CLOSED_IDS = ["15", "18", "28"];
const AUCKLAND_BOUNDS = { minLon: 174, maxLon: 176, minLat: -38, maxLat: -36 };

const isTrue = (value: unknown) => value === true || value === "true";

function parseCsv(text: string): Record<string, string>[] {
  const lines = text
    .replace(/\r/g, "")
    .split("\n")
    .filter((l) => l.length > 0);
  const splitRow = (row: string): string[] => {
    const cells: string[] = [];
    let cell = "";
    let inQuotes = false;
    for (let i = 0; i < row.length; i++) {
      const ch = row[i];
      if (ch === '"') {
        if (inQuotes && row[i + 1] === '"') {
          cell += '"';
          i++;
        } else {
          inQuotes = !inQuotes;
        }
      } else if (ch === "," && !inQuotes) {
        cells.push(cell);
        cell = "";
      } else {
        cell += ch;
      }
    }
    cells.push(cell);
    return cells;
  };
  const header = splitRow(lines[0]);
  return lines.slice(1).map((line) => {
    const cells = splitRow(line);
    return Object.fromEntries(header.map((h, i) => [h, cells[i] ?? ""]));
  });
}

test.describe.serial(
  "Cafe clean-and-export (UI-built) pipeline",
  { tag: "@pipeline" },
  () => {
    test.describe.configure({ timeout: 900_000 });

    const projectName = uniqueProjectName("cafe-clean");
    const deploymentDescription = uniqueDeploymentDescription("cafe-clean");

    let session: EditorSession;
    let page: Page;
    let assets: AssetsPage;
    let projects: ProjectsPage;
    let editor: EditorPage;
    let deployments: DeploymentsPage;

    let cafesUrl: string;
    let cafesAssetName: string;

    let csvReader: Locator;
    let featureFilter: Locator;
    let attributeManager: Locator;
    let geoJsonWriter: Locator;
    let csvWriter: Locator;

    let geojson: any;

    test.beforeAll(async ({ browser }) => {
      session = await newEditorSession(browser);
      ({ page, assets, projects, editor, deployments } = session);
    });

    test.afterAll(async () => {
      await teardownSession(session, {
        projectName,
        deploymentDescription,
        assetNames: [cafesAssetName],
      });
    });

    test("uploads the cafe/restaurant CSV to the workspace", async () => {
      await assets.goto();

      const cafes = await assets.upload(CAFES_CSV);
      cafesUrl = cafes.url;
      cafesAssetName = cafes.name;

      expect(cafesUrl).toMatch(/^https?:\/\//);
    });

    test("creates a new project and opens the editor", async () => {
      await projects.goto();
      await projects.createProject(
        projectName,
        "created by e2e cafe clean-export ui-built test",
      );
      await projects.search(projectName);
      await projects.openProject(projectName);
      await editor.waitForLoaded();

      expect(await editor.isInEditor()).toBe(true);
    });

    test("builds and configures the five nodes", async () => {
      csvReader = await editor.addActionNodeAndGet(
        "reader",
        "CsvReader",
        await editor.canvasPoint(0.5, 0.5),
      );
      featureFilter = await editor.addActionNodeAndGet(
        "transformer",
        "FeatureFilter",
        await editor.canvasPoint(0.1, 0.18),
      );
      attributeManager = await editor.addActionNodeAndGet(
        "transformer",
        "AttributeManager",
        await editor.canvasPoint(0.82, 0.18),
      );
      geoJsonWriter = await editor.addActionNodeAndGet(
        "writer",
        "GeoJsonWriter",
        await editor.canvasPoint(0.82, 0.82),
      );
      csvWriter = await editor.addActionNodeAndGet(
        "writer",
        "CsvWriter",
        await editor.canvasPoint(0.1, 0.82),
      );
      await expect(editor.nodes).toHaveCount(5);
      await editor.openNodeParamsForm(csvReader);
      await editor.setParamSelect(
        "File Format",
        "CSV (Comma-Separated Values)",
      );
      await editor.setParamCodeString("File Path", cafesUrl);
      await editor.setCsvCoordinateGeometry("longitude", "latitude", 4326);
      await editor.submitParams();
      await editor.openNodeParamsForm(featureFilter);
      await editor.addParamArrayItem();
      await editor.setParamFlowExpr(
        "Condition expression",
        'attributes["is_closed"] == "False"',
      );
      await editor.setParamText("root_conditions_0_outputPort", OPEN_PORT);
      await editor.submitParams();
      await editor.openNodeParamsForm(attributeManager);
      await editor.addParamArrayItem();
      await editor.setParamText("root_operations_0_attribute", HIGH_RATED_ATTR);
      await editor.setParamSelect("Operation to perform", "create");
      await editor.setParamFlowExpr(
        "Value",
        'float(attributes["rating"]) >= 4.0',
      );
      await editor.submitParams();
      await editor.openNodeParamsForm(geoJsonWriter);
      await editor.setParamCodeString("output", GEOJSON_OUTPUT);
      await editor.submitParams();

      await editor.openNodeParamsForm(csvWriter);
      await editor.setParamSelect("format", "CSV (Comma-Separated Values)");
      await editor.setCsvCoordinateGeometry("longitude", "latitude");
      await editor.setParamCodeString("output", CSV_OUTPUT);
      await editor.submitParams();

      await editor.connectFromPort(csvReader, featureFilter, "default");
      await editor.connectFromPort(
        featureFilter,
        attributeManager,
        OPEN_PORT,
        "default",
      );
      await editor.connectFromPort(
        attributeManager,
        geoJsonWriter,
        "default",
        "default",
      );
      await editor.connectFromPort(
        attributeManager,
        csvWriter,
        "default",
        "default",
      );
      await expect(editor.edges).toHaveCount(4);
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

    test("exports a GeoJSON of open shops, each flagged and within Auckland", async () => {
      const outputUrl = jobDetailsArtifact(page, GEOJSON_OUTPUT);
      await expect(outputUrl).toBeVisible({ timeout: 90_000 });
      const artifactUrl = (await outputUrl.textContent())?.trim() ?? "";
      test.info().annotations.push({
        type: "geojson-url",
        description: artifactUrl,
      });

      const response = await page.request.get(artifactUrl);
      expect(response.ok()).toBeTruthy();
      geojson = await response.json();
      expect(geojson.type).toBe("FeatureCollection");
      const passed = geojson.features.length;
      test.info().annotations.push({
        type: "filter-count",
        description: `${passed} / ${TOTAL_ROWS} rows kept (open only)`,
      });
      expect(passed).toBe(OPEN_ROWS);

      const ids = new Set<string>();
      let minLon = Infinity,
        maxLon = -Infinity,
        minLat = Infinity,
        maxLat = -Infinity;
      let highRated = 0;
      for (const feature of geojson.features) {
        const props = feature.properties ?? {};
        ids.add(String(props.id));
        expect(props.is_closed).toBe("False");
        const expected = parseFloat(props.rating) >= 4.0;
        expect(isTrue(props[HIGH_RATED_ATTR])).toBe(expected);
        if (isTrue(props[HIGH_RATED_ATTR])) highRated++;
        expect(feature.geometry.type).toBe("Point");
        const [lon, lat] = feature.geometry.coordinates;
        minLon = Math.min(minLon, lon);
        maxLon = Math.max(maxLon, lon);
        minLat = Math.min(minLat, lat);
        maxLat = Math.max(maxLat, lat);
      }

      expect(highRated).toBe(HIGH_RATED_ROWS);
      expect(minLon).toBeGreaterThanOrEqual(AUCKLAND_BOUNDS.minLon);
      expect(maxLon).toBeLessThanOrEqual(AUCKLAND_BOUNDS.maxLon);
      expect(minLat).toBeGreaterThanOrEqual(AUCKLAND_BOUNDS.minLat);
      expect(maxLat).toBeLessThanOrEqual(AUCKLAND_BOUNDS.maxLat);
      for (const id of HIGH_RATED_BUT_CLOSED_IDS)
        expect(ids.has(id)).toBe(false);
    });

    test("exports a matching CSV with the high_rated column", async () => {
      const outputUrl = jobDetailsArtifact(page, CSV_OUTPUT);
      await expect(outputUrl).toBeVisible({ timeout: 90_000 });
      const artifactUrl = (await outputUrl.textContent())?.trim() ?? "";
      test.info().annotations.push({
        type: "csv-url",
        description: artifactUrl,
      });

      const response = await page.request.get(artifactUrl);
      expect(response.ok()).toBeTruthy();
      const rows = parseCsv(await response.text());
      expect(rows).toHaveLength(OPEN_ROWS);
      expect(rows.every((r) => r.is_closed === "False")).toBe(true);
      const header = Object.keys(rows[0]);
      expect(header).toContain(HIGH_RATED_ATTR);
      expect(header).toContain("longitude");
      expect(header).toContain("latitude");

      const highRated = rows.filter((r) => isTrue(r[HIGH_RATED_ATTR])).length;
      expect(highRated).toBe(HIGH_RATED_ROWS);
    });
  },
);
