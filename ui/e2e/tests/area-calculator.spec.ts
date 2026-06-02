import * as path from "path";

import { expect, test } from "@playwright/test";

import {
  DeploymentsPage,
  uniqueDeploymentDescription,
} from "../pages/deploymentsPage";

const WORKFLOW = path.resolve(
  __dirname,
  "fixtures/workflows/area-calculator.yml",
);

test.describe("Area calculator pipeline", { tag: "@pipeline" }, () => {
  let deployments: DeploymentsPage;
  let description: string;

  test.beforeEach(async ({ page }) => {
    deployments = new DeploymentsPage(page);
    description = uniqueDeploymentDescription("area-calc");
    await deployments.goto();
  });

  test.afterEach(async () => {
    await deployments.goto();
    await deployments.deleteDeploymentIfExists(description).catch(() => { });
  });

  test("deploys, runs to completion, and produces the park-areas artifact", async ({
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
    console.log("JOB_URL:", page.url());

    // Wait for a terminal status; engine cold start can take minutes.
    const terminalStatus = page
      .getByText(/^(completed|failed|cancelled)$/)
      .first();
    await expect(terminalStatus).toBeVisible({ timeout: 300_000 });
    await expect(terminalStatus).toHaveText("completed");

    // The output artifact is listed asynchronously after completion.
    const outputUrl = page.getByText(/park-areas\.geojson/).first();
    await expect(outputUrl).toBeVisible({ timeout: 90_000 });
    console.log("OUTPUT_URL:", (await outputUrl.textContent())?.trim());
  });
});
