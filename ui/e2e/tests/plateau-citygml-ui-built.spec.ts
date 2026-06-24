import { expect, test, type Locator, type Page } from "@playwright/test";

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

const CITYGML_URL =
  "https://assets.cms.plateau.reearth.io/assets/45/40a1ee-1e80-4d69-bc6d-d75b77033151/13362_toshima-mura_pref_2024_citygml_1_op.zip";

test.describe.serial(
  "PLATEAU CityGML (UI-built) pipeline",
  { tag: "@pipeline" },
  () => {
    test.describe.configure({ timeout: 1_500_000 });

    const projectName = uniqueProjectName("plateau-citygml-ui");
    const deploymentDescription =
      uniqueDeploymentDescription("plateau-citygml-ui");

    let session: EditorSession;
    let page: Page;
    let projects: ProjectsPage;
    let editor: EditorPage;
    let deployments: DeploymentsPage;

    let fileExtractor: Locator;
    let filterGml: Locator;
    let filterBuilding: Locator;
    let cityGmlReader: Locator;
    let attributeMapper: Locator;
    let geoJsonWriter: Locator;

    test.beforeAll(async ({ browser }) => {
      session = await newEditorSession(browser);
      ({ page, projects, editor, deployments } = session);
    });

    test.afterAll(async () => {
      await teardownSession(session, { projectName, deploymentDescription });
    });

    test("creates a new project and opens the editor", async () => {
      await projects.goto();
      await projects.createProject(
        projectName,
        "created by e2e plateau ui-built test",
      );
      await projects.search(projectName);
      await projects.openProject(projectName);
      await editor.waitForLoaded();

      expect(await editor.isInEditor()).toBe(true);
    });

    test("builds and configures the six available nodes", async () => {
      fileExtractor = await editor.addActionNodeAndGet(
        "reader",
        "FilePathExtractor",
        await editor.canvasPoint(0.15, 0.25),
      );
      filterGml = await editor.addActionNodeAndGet(
        "transformer",
        "FeatureFilter",
        await editor.canvasPoint(0.42, 0.22),
      );
      filterBuilding = await editor.addActionNodeAndGet(
        "transformer",
        "FeatureFilter",
        await editor.canvasPoint(0.72, 0.22),
      );
      cityGmlReader = await editor.addActionNodeAndGet(
        "transformer",
        "FeatureCityGmlReader",
        await editor.canvasPoint(0.72, 0.72),
      );
      attributeMapper = await editor.addActionNodeAndGet(
        "transformer",
        "AttributeMapper",
        await editor.canvasPoint(0.42, 0.72),
      );
      geoJsonWriter = await editor.addActionNodeAndGet(
        "writer",
        "GeoJsonWriter",
        await editor.canvasPoint(0.15, 0.72),
      );
      await expect(editor.nodes).toHaveCount(6);

      await editor.openNodeParamsForm(fileExtractor);
      await editor.setParamCodeString("Source Dataset", CITYGML_URL);
      await editor.setParamCheckbox("root_extractArchive", true);
      await editor.submitParams();

      await editor.openNodeParamsForm(filterGml);
      await editor.addParamArrayItem();
      await editor.setParamFlowExpr(
        "Condition expression",
        'attributes["extension"] == "gml"',
      );
      await editor.setParamText("root_conditions_0_outputPort", "default");
      await editor.submitParams();

      await editor.openNodeParamsForm(filterBuilding);
      await editor.addParamArrayItem();
      await editor.setParamFlowExpr(
        "Condition expression",
        'attributes["package"] == "bldg"',
      );
      await editor.setParamText("root_conditions_0_outputPort", "default");
      await editor.submitParams();

      await editor.openNodeParamsForm(cityGmlReader);
      await editor.setParamFlowExpr("Dataset", 'env.get("__value")["path"]');
      await editor.submitParams();

      await editor.openNodeParamsForm(attributeMapper);
      for (let i = 0; i < 4; i++) await editor.addParamArrayItem();
      await editor.setParamText("root_mappers_0_attribute", "gmlId");
      await editor.setParamText("root_mappers_0_valueAttribute", "gmlId");
      await editor.setParamText("root_mappers_1_attribute", "featureType");
      await editor.setParamText("root_mappers_1_valueAttribute", "featureType");
      await editor.setParamText("root_mappers_2_attribute", "maxLod");
      await editor.setParamText("root_mappers_2_valueAttribute", "maxLod");
      await editor.setParamText("root_mappers_3_attribute", "meshcode");
      await editor.setParamFlowExpr(
        "Expression to evaluate",
        'attributes["path"].split("/")[-1].split("_")[0]',
        3,
      );
      await editor.submitParams();

      await editor.openNodeParamsForm(geoJsonWriter);
      await editor.setParamCodeString("output", "toshima-buildings.geojson");
      await editor.submitParams();

      await editor.connectFromPort(fileExtractor, filterGml, "default");
      await editor.connectFromPort(filterBuilding, cityGmlReader, "default");
      await editor.connectFromPort(cityGmlReader, attributeMapper, "default");
      await editor.connectFromPort(attributeMapper, geoJsonWriter, "default");
      await expect(editor.edges).toHaveCount(4);
    });

    test("PLATEAU4.UDXFolderExtractor is absent from the dev catalog (canary)", async () => {
      await editor.dragToolToCanvas(
        "transformer",
        await editor.canvasPoint(0.5, 0.85),
      );
      await expect(editor.actionPicker).toBeVisible();
      await editor.actionPicker
        .getByPlaceholder(/^Search/)
        .fill("UDXFolderExtractor");
      await expect(
        editor.actionPicker
          .locator("span")
          .filter({ hasText: /UDXFolderExtractor/ }),
      ).toHaveCount(0);
      await page.keyboard.press("Escape");
      await expect(editor.actionPicker).toBeHidden();
    });

    // fixme: PLATEAU4.UDXFolderExtractor is not yet in the dev catalog (asserted
    // absent by the canary test above); enable once it ships to dev.
    test.fixme("completes the chain with UDX, deploys, runs, and produces the buildings artifact", async () => {
      const udxExtractor = await editor.addActionNodeAndGet(
        "transformer",
        "PLATEAU4.UDXFolderExtractor",
        await editor.canvasPoint(0.5, 0.47),
      );
      await editor.openNodeParamsForm(udxExtractor);
      await editor.setParamText(
        "root_cityGmlPath",
        'env.get("__value")["path"]',
      );
      await editor.submitParams();

      await editor.connectFromPort(filterGml, udxExtractor, "default");
      await editor.connectFromPort(udxExtractor, filterBuilding, "default");
      await expect(editor.edges).toHaveCount(6);

      await editor.deploy(deploymentDescription);

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

      await expectJobSucceeded(page, 1_380_000);

      const outputUrl = jobDetailsArtifact(page, "toshima-buildings.geojson");
      await expect(outputUrl).toBeVisible({ timeout: 90_000 });
      const artifactUrl = (await outputUrl.textContent())?.trim() ?? "";
      test.info().annotations.push({
        type: "output-url",
        description: artifactUrl,
      });

      const response = await page.request.get(artifactUrl);
      expect(response.ok()).toBeTruthy();
      const geojson = await response.json();
      expect(geojson.type).toBe("FeatureCollection");
      expect(geojson.features.length).toBeGreaterThan(0);
      expect(geojson.features[0].properties.featureType).toBe("bldg:Building");
    });
  },
);
