import { Locator, Page } from "@playwright/test";

type SidebarItem =
  | "Projects"
  | "Deployments"
  | "Triggers"
  | "Jobs"
  | "Workspace Assets"
  | "General Settings"
  | "Member Settings";

export class HomePage {
  readonly brandText: Locator;
  readonly currentWorkspaceLabel: Locator;
  readonly workspaceSwitcher: Locator;
  readonly accountMenuTrigger: Locator;
  readonly projectsHeading: Locator;
  readonly newProjectButton: Locator;
  readonly importButton: Locator;
  readonly searchInput: Locator;
  readonly sortSelector: Locator;
  readonly emptyProjectsState: Locator;

  constructor(private page: Page) {
    this.brandText = page.getByText("Re:Earth Flow", { exact: true });
    this.currentWorkspaceLabel = page.getByText("Current workspace:");
    this.workspaceSwitcher = page
      .locator('button[data-slot="dropdown-menu-trigger"]')
      .filter({ hasText: "Current workspace:" });
    this.accountMenuTrigger = page
      .locator('button[data-slot="dropdown-menu-trigger"]')
      .first();
    this.projectsHeading = page.locator("p.text-lg", { hasText: "Projects" });
    this.newProjectButton = page.getByRole("button", { name: "New Project" });
    this.importButton = page.getByRole("button", { name: "Import" });
    this.searchInput = page.getByPlaceholder("Search...");
    this.sortSelector = page.getByRole("combobox");
    this.emptyProjectsState = page.getByText("No Projects");
  }

  sidebarItem(name: SidebarItem): Locator {
    return this.page
      .locator("div.cursor-pointer")
      .filter({
        has: this.page.locator("p", { hasText: new RegExp(`^${name}$`) }),
      })
      .first();
  }

  async waitForLoaded() {
    await this.brandText.waitFor({ state: "visible", timeout: 30_000 });
    await this.currentWorkspaceLabel.waitFor({
      state: "visible",
      timeout: 30_000,
    });
  }

  async isLoaded() {
    return this.currentWorkspaceLabel.isVisible().catch(() => false);
  }

  async navigateTo(item: SidebarItem) {
    await this.sidebarItem(item).click();
  }
}
