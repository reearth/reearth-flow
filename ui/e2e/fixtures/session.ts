import type { Browser, BrowserContext, Page } from "@playwright/test";

import { AssetsPage } from "../pages/assetsPage";
import { DeploymentsPage } from "../pages/deploymentsPage";
import { EditorPage } from "../pages/editorPage";
import { ProjectsPage } from "../pages/projectsPage";
import { STORAGE_STATE } from "../playwright.config";

export type EditorSession = {
  context: BrowserContext;
  page: Page;
  assets: AssetsPage;
  projects: ProjectsPage;
  editor: EditorPage;
  deployments: DeploymentsPage;
};

export async function newEditorSession(
  browser: Browser,
): Promise<EditorSession> {
  const context = await browser.newContext({
    storageState: STORAGE_STATE,
    baseURL: process.env.FLOW_DASHBOARD_E2E_BASEURL,
    viewport: { width: 1920, height: 1080 },
    locale: "en-US",
  });
  const page = await context.newPage();
  return {
    context,
    page,
    assets: new AssetsPage(page),
    projects: new ProjectsPage(page),
    editor: new EditorPage(page),
    deployments: new DeploymentsPage(page),
  };
}

export async function teardownSession(
  session: EditorSession | undefined,
  opts: {
    projectName?: string;
    deploymentDescription?: string;
    assetNames?: (string | undefined)[];
  } = {},
) {
  if (!session) return;
  const { context, assets, projects, deployments } = session;
  try {
    if (opts.deploymentDescription) {
      await deployments.goto();
      await deployments
        .deleteDeploymentIfExists(opts.deploymentDescription)
        .catch(() => { });
    }
    if (opts.projectName) {
      await projects.goto();
      await projects.deleteProjectIfExists(opts.projectName).catch(() => { });
    }
    for (const name of opts.assetNames ?? []) {
      if (!name) continue;
      await assets.goto();
      await assets.deleteAssetIfExists(name).catch(() => { });
    }
  } finally {
    await context.close();
  }
}
