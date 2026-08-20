import * as path from "path";

import { expect, test } from "@playwright/test";

import {
  DeploymentsPage,
  uniqueDeploymentDescription,
} from "../pages/deploymentsPage";

const WORKFLOW = path.resolve(
  __dirname,
  "fixtures/workflows/plateau-citygml.yml",
);

test.describe("PLATEAU CityGML pipeline", { tag: "@pipeline" }, () => {
  let deployments: DeploymentsPage;
  let description: string;

  test.beforeEach(async ({ page }) => {
    deployments = new DeploymentsPage(page);
    description = uniqueDeploymentDescription("plateau-citygml");
    await deployments.goto();
  });

  test.afterEach(async () => {
    await deployments.goto();
    await deployments.deleteDeploymentIfExists(description).catch(() => {});
  });

  // fixme: this workflow uses PLATEAU4.UDXFolderExtractor, which is not yet in
  // the dev engine catalog (the UI-built sibling asserts its absence and skips
  // the UDX chain for the same reason). Re-enable once UDXFolderExtractor ships
  // to the dev worker.
  test.fixme("deploys, processes the Toshima-mura city model, and produces the buildings artifact", async ({
    page,
  }) => {
    test.setTimeout(1_500_000);

    await deployments.createFromFile(WORKFLOW, description);

    await deployments.search(description);
    await deployments.openDetails(description);
    await deployments.runFromDetails();

    await expect(page).toHaveURL(/\/workspaces\/[^/]+\/jobs\/[^/]+$/, {
      timeout: 15_000,
    });
    test.info().annotations.push({ type: "job-url", description: page.url() });

    const terminalStatus = page
      .getByText(/^(completed|failed|cancelled)$/)
      .first();
    await expect(terminalStatus).toBeVisible({ timeout: 1_380_000 });
    await expect(terminalStatus).toHaveText("completed");

    const outputUrl = page.getByText(/toshima-buildings\.geojson/).first();
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
});
