import { expect, Locator, Page } from "@playwright/test";

export type ToolId =
  | "reader"
  | "transformer"
  | "writer"
  | "note"
  | "batch"
  | "subworkflow";

export type ActionToolId = "reader" | "transformer" | "writer";

type Point = { x: number; y: number };

export class EditorPage {
  readonly canvas: Locator;
  readonly loadingSplash: Locator;
  readonly nodes: Locator;
  readonly edges: Locator;
  readonly actionPicker: Locator;
  readonly paramsDialog: Locator;
  readonly confirmDialog: Locator;
  readonly deployButton: Locator;
  readonly deployPopover: Locator;
  readonly deployDescriptionInput: Locator;
  readonly deploySubmitButton: Locator;
  readonly deploymentCreatedToast: Locator;
  readonly deploymentUpdatedToast: Locator;

  constructor(private page: Page) {
    this.canvas = page.locator(".react-flow");
    this.loadingSplash = page.getByText("Re:Earth Flow", { exact: true });
    this.nodes = page.locator(".react-flow__node");
    this.edges = page.locator(".react-flow__edge");
    this.actionPicker = page
      .getByRole("dialog")
      .filter({ hasText: "Choose Action" });
    this.paramsDialog = page
      .getByRole("dialog")
      .filter({ hasText: "Action Editor" });
    this.confirmDialog = page.getByRole("alertdialog");
    this.deployButton = page
      .locator("#right-top > div > div")
      .last()
      .locator("button")
      .first();
    this.deployPopover = page
      .getByRole("dialog")
      .filter({ hasText: "Deploy Project" });
    this.deployDescriptionInput = this.deployPopover.getByPlaceholder(
      "Give your deployment a meaningful description...",
    );
    this.deploySubmitButton = this.deployPopover.getByRole("button", {
      name: /^(Deploy|Update)$/,
    });
    this.deploymentCreatedToast = page.getByText("Deployment Created", {
      exact: true,
    });
    this.deploymentUpdatedToast = page.getByText("Deployment Updated", {
      exact: true,
    });
  }

  toolButton(tool: ToolId): Locator {
    return this.page.locator(`button.dndnode-${tool}`);
  }

  nodeByName(name: string): Locator {
    return this.nodes.filter({ hasText: name });
  }

  async waitForLoaded() {
    await this.canvas.waitFor({ state: "visible", timeout: 60_000 });
    await this.toolButton("transformer").waitFor({
      state: "visible",
      timeout: 30_000,
    });
  }

  async isInEditor(): Promise<boolean> {
    return /\/projects\/[^/]+$/.test(new URL(this.page.url()).pathname);
  }

  async nodeCount(): Promise<number> {
    return this.nodes.count();
  }

  async edgeCount(): Promise<number> {
    return this.edges.count();
  }

  async canvasPoint(fractionX = 0.5, fractionY = 0.5): Promise<Point> {
    const box = await this.canvas.boundingBox();
    if (!box) throw new Error("Canvas is not visible");
    return {
      x: box.x + box.width * fractionX,
      y: box.y + box.height * fractionY,
    };
  }

  async dragToolToCanvas(tool: ToolId, target?: Point) {
    const source = this.toolButton(tool);
    await source.waitFor({ state: "visible" });
    const point = target ?? (await this.canvasPoint());

    const dataTransfer = await this.page.evaluateHandle(
      () => new DataTransfer(),
    );
    await source.dispatchEvent("dragstart", { dataTransfer });
    await this.canvas.dispatchEvent("dragenter", {
      dataTransfer,
      clientX: point.x,
      clientY: point.y,
    });
    await this.canvas.dispatchEvent("dragover", {
      dataTransfer,
      clientX: point.x,
      clientY: point.y,
    });
    await this.canvas.dispatchEvent("drop", {
      dataTransfer,
      clientX: point.x,
      clientY: point.y,
    });
    await source.dispatchEvent("dragend", { dataTransfer });
    await dataTransfer.dispose();
  }

  async addActionNode(tool: ActionToolId, target?: Point): Promise<string> {
    const before = await this.nodes.count();
    await this.dragToolToCanvas(tool, target);
    await expect(this.actionPicker).toBeVisible();

    const firstAction = this.actionPicker
      .locator("span.flex-1.truncate.text-sm")
      .first();
    await expect(firstAction).toBeVisible();
    const name = (await firstAction.textContent())?.trim();
    if (!name) {
      throw new Error("Action picker has no labelled actions to choose from");
    }
    await firstAction.dblclick();

    await expect(this.actionPicker).toBeHidden();
    await expect(this.nodes).toHaveCount(before + 1);
    return name;
  }

  async connectNodes(source: Locator, target: Locator) {
    const sourceHandle = source.locator(".react-flow__handle-right").first();
    const targetHandle = target.locator(".react-flow__handle-left").first();
    const from = await sourceHandle.boundingBox();
    const to = await targetHandle.boundingBox();
    if (!from || !to) throw new Error("Connection handles not found");

    const fromX = from.x + from.width / 2;
    const fromY = from.y + from.height / 2;
    const toX = to.x + to.width / 2;
    const toY = to.y + to.height / 2;

    await this.page.mouse.move(fromX, fromY);
    await this.page.mouse.down();
    await this.page.mouse.move((fromX + toX) / 2, (fromY + toY) / 2, {
      steps: 8,
    });
    await this.page.mouse.move(toX, toY, { steps: 8 });
    await this.page.mouse.up();
  }

  async selectNode(node: Locator) {
    await node.click();
  }

  async clickPane() {
    const box = await this.canvas.boundingBox();
    if (!box) throw new Error("Canvas is not visible");
    await this.page.mouse.click(box.x + 40, box.y + box.height - 40);
  }

  async openNodeParams(node: Locator) {
    await node.dblclick();
    await expect(this.paramsDialog).toBeVisible();
  }

  async setNodeCustomization(node: Locator, value: string): Promise<string> {
    await this.openNodeParams(node);
    await this.paramsDialog
      .getByRole("tab", { name: "Customizations" })
      .click();
    const field = this.paramsDialog.getByRole("textbox").first();
    await field.waitFor({ state: "visible" });
    await field.fill(value);
    await this.paramsDialog.getByRole("button", { name: "Update" }).click();
    await expect(this.paramsDialog).toBeHidden();
    return value;
  }

  async readNodeCustomization(node: Locator): Promise<string> {
    await this.openNodeParams(node);
    await this.paramsDialog
      .getByRole("tab", { name: "Customizations" })
      .click();
    const field = this.paramsDialog.getByRole("textbox").first();
    await field.waitFor({ state: "visible" });
    return field.inputValue();
  }

  async closeParamsDialog() {
    await this.page.keyboard.press("Escape");
    await expect(this.paramsDialog).toBeHidden();
  }

  async deleteSelected() {
    await this.page.keyboard.press("Backspace");
    const confirmed = await this.confirmDialog
      .waitFor({ state: "visible", timeout: 3000 })
      .then(() => true)
      .catch(() => false);
    if (confirmed) {
      await this.confirmDialog
        .getByRole("button", { name: "Continue" })
        .click();
      await expect(this.confirmDialog).toBeHidden();
    }
  }

  async undo() {
    await this.page.keyboard.press("ControlOrMeta+z");
  }

  async redo() {
    await this.page.keyboard.press("ControlOrMeta+Shift+z");
  }

  async selectAll() {
    await this.page.keyboard.press("ControlOrMeta+a");
  }

  async copySelected() {
    await this.page.keyboard.press("ControlOrMeta+c");
  }

  contextMenuItem(label: string): Locator {
    return this.page.getByText(label, { exact: true });
  }

  async openNodeContextMenu(node: Locator) {
    await node.click({ button: "right" });
  }

  async openPaneContextMenu(point?: Point) {
    const target = point ?? (await this.canvasPoint(0.25, 0.75));
    await this.page.mouse.click(target.x, target.y, { button: "right" });
  }

  async copyNode(node: Locator) {
    await this.openNodeContextMenu(node);
    await this.contextMenuItem("Copy").click();
  }

  async pasteAtPane(point?: Point) {
    await this.openPaneContextMenu(point);
    await this.contextMenuItem("Paste").click();
  }

  async openSubworkflow(node: Locator) {
    await this.openNodeContextMenu(node);
    await this.contextMenuItem("Open Subworkflow").click();
  }

  async openDeployPopover() {
    await this.deployButton.click();
    await expect(this.deployDescriptionInput).toBeVisible();
  }

  async deploy(description: string) {
    await this.openDeployPopover();
    await this.deployDescriptionInput.fill(description);
    await this.deploySubmitButton.click();
    await expect(this.deploymentCreatedToast).toBeVisible({ timeout: 30_000 });
    await expect(this.deployDescriptionInput).toBeHidden();
  }

  async updateDeployment(newDescription: string) {
    await this.openDeployPopover();
    await expect(this.deploySubmitButton).toHaveText("Update", {
      timeout: 15_000,
    });
    await this.deployDescriptionInput.fill(newDescription);
    await this.deploySubmitButton.click();
    await expect(this.deploymentUpdatedToast).toBeVisible({ timeout: 30_000 });
    await expect(this.deployDescriptionInput).toBeHidden();
  }
}
