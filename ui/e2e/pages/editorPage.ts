import { Locator, Page } from "@playwright/test";

export class EditorPage {
  readonly canvas: Locator;
  readonly loadingSplash: Locator;
  readonly nodes: Locator;

  constructor(private page: Page) {
    this.canvas = page.locator(".react-flow");
    this.loadingSplash = page.getByText("Re:Earth Flow", { exact: true });
    this.nodes = page.locator(".react-flow__node");
  }

  nodeByName(name: string): Locator {
    return this.page.locator(".react-flow__node").filter({ hasText: name });
  }

  async waitForLoaded() {
    await this.canvas.waitFor({ state: "visible", timeout: 60_000 });
  }

  async isInEditor(): Promise<boolean> {
    return /\/projects\/[^/]+$/.test(new URL(this.page.url()).pathname);
  }
}
