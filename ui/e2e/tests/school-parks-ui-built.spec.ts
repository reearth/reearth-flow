import * as fs from "fs";
import * as path from "path";

import {
  expect,
  test,
  type BrowserContext,
  type Locator,
  type Page,
} from "@playwright/test";

import { AssetsPage } from "../pages/assetsPage";
import {
  DeploymentsPage,
  uniqueDeploymentDescription,
} from "../pages/deploymentsPage";
import { EditorPage } from "../pages/editorPage";
import { ProjectsPage, uniqueProjectName } from "../pages/projectsPage";
import { STORAGE_STATE } from "../playwright.config";

const SCHOOLS_ZIP = path.resolve(
  __dirname,
  "../assets/Schools_directory_NZ_modified.zip",
);
const PARKS_GEOJSON = path.resolve(__dirname, "../assets/Park_Extents.geojson");

// Schools and parks are both EPSG:4326 (degrees), so Bufferer.distance is in
// degrees: ~0.0045° ≈ 500 m at Auckland's latitude (the "500-meter buffer" of
// the scenario). Tune up slightly if a run finds zero overlapping parks.
const BUFFER_DISTANCE = "0.0045";
const INTERPOLATION_ANGLE = "10";
const OUTPUT_FILE = "parks-near-schools.geojson";
// SpatialFilter annotates each passed park with how many school buffers it
// matched, so the test can prove the filter did real spatial work.
const MATCH_COUNT_ATTR = "schoolBufferMatches";
// Generous New Zealand lon/lat box — confirms output coordinates are real
// WGS84 park geometry, not corrupted, empty, or reprojected to metres.
const NZ_BOUNDS = { minLon: 166, maxLon: 179, minLat: -48, maxLat: -34 };

function* coordsOf(geometry: any): Generator<[number, number]> {
  const polygons =
    geometry.type === "MultiPolygon"
      ? geometry.coordinates
      : [geometry.coordinates];
  for (const rings of polygons)
    for (const ring of rings)
      for (const position of ring) yield [position[0], position[1]];
}

test.describe.serial(
  "School-to-parks accessibility (UI-built) pipeline",
  { tag: "@pipeline" },
  () => {
    test.describe.configure({ timeout: 900_000 });

    const projectName = uniqueProjectName("school-parks");
    const deploymentDescription = uniqueDeploymentDescription("school-parks");

    let context: BrowserContext;
    let page: Page;
    let assets: AssetsPage;
    let projects: ProjectsPage;
    let editor: EditorPage;
    let deployments: DeploymentsPage;

    let schoolsUrl: string;
    let parksUrl: string;
    let schoolsAssetName: string;
    let parksAssetName: string;

    let shapefileReader: Locator;
    let bufferer: Locator;
    let geoJsonReader: Locator;
    let spatialFilter: Locator;
    let geoJsonWriter: Locator;

    let geojson: any;

    test.beforeAll(async ({ browser }) => {
      context = await browser.newContext({
        storageState: STORAGE_STATE,
        baseURL: process.env.FLOW_DASHBOARD_E2E_BASEURL,
        viewport: { width: 1920, height: 1080 },
        locale: "en-US",
      });
      page = await context.newPage();
      assets = new AssetsPage(page);
      projects = new ProjectsPage(page);
      editor = new EditorPage(page);
      deployments = new DeploymentsPage(page);
    });

    test.afterAll(async () => {
      if (!context) return;
      try {
        await deployments.goto();
        await deployments
          .deleteDeploymentIfExists(deploymentDescription)
          .catch(() => {});
        await projects.goto();
        await projects.deleteProjectIfExists(projectName).catch(() => {});
        await assets.goto();
        if (schoolsAssetName)
          await assets.deleteAssetIfExists(schoolsAssetName).catch(() => {});
        if (parksAssetName)
          await assets.deleteAssetIfExists(parksAssetName).catch(() => {});
      } finally {
        await context.close();
      }
    });

    test("uploads the schools and parks datasets to the workspace", async () => {
      await assets.goto();

      const schools = await assets.upload(SCHOOLS_ZIP);
      schoolsUrl = schools.url;
      schoolsAssetName = schools.name;

      const parks = await assets.upload(PARKS_GEOJSON);
      parksUrl = parks.url;
      parksAssetName = parks.name;

      expect(schoolsUrl).toMatch(/^https?:\/\//);
      expect(parksUrl).toMatch(/^https?:\/\//);
    });

    test("creates a new project and opens the editor", async () => {
      await projects.goto();
      await projects.createProject(
        projectName,
        "created by e2e school-parks ui-built test",
      );
      await projects.search(projectName);
      await projects.openProject(projectName);
      await editor.waitForLoaded();

      expect(await editor.isInEditor()).toBe(true);
    });

    test("builds and configures the five nodes", async () => {
      shapefileReader = await editor.addActionNodeAndGet(
        "reader",
        "ShapefileReader",
        await editor.canvasPoint(0.12, 0.25),
      );
      bufferer = await editor.addActionNodeAndGet(
        "transformer",
        "Bufferer",
        await editor.canvasPoint(0.4, 0.25),
      );
      geoJsonReader = await editor.addActionNodeAndGet(
        "reader",
        "GeoJsonReader",
        await editor.canvasPoint(0.12, 0.7),
      );
      spatialFilter = await editor.addActionNodeAndGet(
        "transformer",
        "SpatialFilter",
        await editor.canvasPoint(0.62, 0.48),
      );
      geoJsonWriter = await editor.addActionNodeAndGet(
        "writer",
        "GeoJsonWriter",
        await editor.canvasPoint(0.86, 0.48),
      );
      await expect(editor.nodes).toHaveCount(5);

      await editor.openNodeParamsForm(shapefileReader);
      await editor.setParamCodeString("File Path", schoolsUrl);
      await editor.submitParams();

      await editor.openNodeParamsForm(bufferer);
      await editor.setParamSelect("Buffer Type", "2D Area Buffer");
      await editor.setParamText("root_distance", BUFFER_DISTANCE);
      await editor.setParamText("root_interpolationAngle", INTERPOLATION_ANGLE);
      await editor.submitParams();

      await editor.openNodeParamsForm(geoJsonReader);
      await editor.setParamCodeString("File Path", parksUrl);
      await editor.submitParams();

      // SpatialFilter keeps its defaults (predicate=intersects, pass-on-any-
      // match) — a park passes when it overlaps any school buffer. The match-
      // count attribute is added only so the test can assert each passed park
      // matched at least one buffer.
      await editor.openNodeParamsForm(spatialFilter);
      await editor.setParamText(
        "root_outputMatchCountAttribute",
        MATCH_COUNT_ATTR,
      );
      await editor.submitParams();

      // Sink outputs must be relative paths; the engine resolves them against
      // the job's artifact sandbox.
      await editor.openNodeParamsForm(geoJsonWriter);
      await editor.setParamCodeString("output", OUTPUT_FILE);
      await editor.submitParams();

      await editor.connectFromPort(shapefileReader, bufferer, "default");
      await editor.connectFromPort(
        bufferer,
        spatialFilter,
        "default",
        "filter",
      );
      await editor.connectFromPort(
        geoJsonReader,
        spatialFilter,
        "default",
        "candidate",
      );
      await editor.connectFromPort(
        spatialFilter,
        geoJsonWriter,
        "passed",
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

      // Job-level status only: the dot in the Job Details box tracks the
      // overall job status, so it can't be tripped early by a per-node line in
      // the log console below it. Wait until it leaves running/queued, then
      // require success.
      const statusDot = page
        .locator("div.rounded-md.border")
        .filter({ hasText: "Job Details" })
        .locator(".size-4.rounded-full");
      await expect(statusDot).toHaveClass(
        /bg-success|bg-destructive|bg-warning/,
        { timeout: 600_000 },
      );
      await expect(statusDot).toHaveClass(/bg-success/);
    });

    test("filters parks down to those within a school buffer, with valid NZ geometry", async () => {
      // Read the artifact URL from the Job Details box, not arbitrary page text,
      // so a log line naming the file can't be mistaken for the output link.
      const outputUrl = page
        .locator("div.rounded-md.border")
        .filter({ hasText: "Job Details" })
        .getByText(/parks-near-schools\.geojson/)
        .first();
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

      // Filter count: some parks pass, but not all — proves the spatial filter
      // excluded parks rather than passing the whole candidate set through.
      const totalParks = JSON.parse(fs.readFileSync(PARKS_GEOJSON, "utf8"))
        .features.length;
      const passed = geojson.features.length;
      test.info().annotations.push({
        type: "filter-count",
        description: `${passed} / ${totalParks} parks within a school buffer`,
      });
      expect(passed).toBeGreaterThan(0);
      expect(passed).toBeLessThan(totalParks);

      // Geometry: every passed park is a polygon the engine reports as
      // overlapping >= 1 school buffer, with coordinates inside New Zealand.
      let minLon = Infinity,
        maxLon = -Infinity,
        minLat = Infinity,
        maxLat = -Infinity;
      for (const feature of geojson.features) {
        expect(feature.geometry.type).toMatch(/Polygon/);
        expect(feature.properties?.[MATCH_COUNT_ATTR]).toBeGreaterThanOrEqual(
          1,
        );
        for (const [lon, lat] of coordsOf(feature.geometry)) {
          minLon = Math.min(minLon, lon);
          maxLon = Math.max(maxLon, lon);
          minLat = Math.min(minLat, lat);
          maxLat = Math.max(maxLat, lat);
        }
      }
      expect(minLon).toBeGreaterThanOrEqual(NZ_BOUNDS.minLon);
      expect(maxLon).toBeLessThanOrEqual(NZ_BOUNDS.maxLon);
      expect(minLat).toBeGreaterThanOrEqual(NZ_BOUNDS.minLat);
      expect(maxLat).toBeLessThanOrEqual(NZ_BOUNDS.maxLat);

      // Passed features are genuinely park records with attributes intact.
      expect(
        geojson.features.some((f: any) => f.properties?.AssetGroup === "Park"),
      ).toBe(true);
    });
  },
);
