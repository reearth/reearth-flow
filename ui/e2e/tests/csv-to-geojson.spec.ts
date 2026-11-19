import * as path from "path";

import { expect, test } from "@playwright/test";

import { expectJobSucceeded } from "../helpers/job";
import {
  DeploymentsPage,
  uniqueDeploymentDescription,
} from "../pages/deploymentsPage";

const WORKFLOW = path.resolve(
  __dirname,
  "fixtures/workflows/csv-to-geojson.yml",
);

test.describe("CSV to GeoJSON pipeline", { tag: "@pipeline" }, () => {
  let deployments: DeploymentsPage;
  let description: string;

  test.beforeEach(async ({ page }) => {
    deployments = new DeploymentsPage(page);
    description = uniqueDeploymentDescription("csv-to-geojson");
    await deployments.goto();
  });

  test.afterEach(async () => {
    await deployments.goto();
    await deployments.deleteDeploymentIfExists(description).catch(() => {});
  });

  test("deploys, runs to completion, and produces the stations artifact", async ({
    page,
  }) => {
    test.setTimeout(360_000);

    await deployments.createFromFile(WORKFLOW, description);

    await deployments.search(description);
    await deployments.openDetails(description);
    await deployments.runFromDetails();

    await expect(page).toHaveURL(/\/workspaces\/[^/]+\/jobs\/[^/]+$/, {
      timeout: 15_000,
    });
    test.info().annotations.push({ type: "job-url", description: page.url() });

    // Wait for a terminal status; engine cold start can take minutes.
    await expectJobSucceeded(page, 300_000);

    // The output artifact is listed asynchronously after completion.
    const outputUrl = page.getByText(/stations\.geojson/).first();
    await expect(outputUrl).toBeVisible({ timeout: 90_000 });
    test.info().annotations.push({
      type: "output-url",
      description: (await outputUrl.textContent())?.trim() ?? "",
    });
  });
});
