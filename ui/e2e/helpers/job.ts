import { expect, type Locator, type Page } from "@playwright/test";

function jobDetailsBox(page: Page): Locator {
  return page
    .locator("div.rounded-md.border")
    .filter({ hasText: "Job Details" });
}

function outputDataBox(page: Page): Locator {
  return page
    .locator("div.rounded-md.border")
    .filter({ hasText: "Output Data" });
}

function escapeRegExp(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

export function jobOutputArtifact(page: Page, fileName: string): Locator {
  return outputDataBox(page)
    .getByRole("link", { name: new RegExp(escapeRegExp(fileName)) })
    .first();
}

export async function jobOutputArtifactUrl(
  page: Page,
  fileName: string,
  timeout = 90_000,
): Promise<string> {
  const link = jobOutputArtifact(page, fileName);
  await expect(link).toBeVisible({ timeout });
  return (await link.getAttribute("href")) ?? "";
}

export async function expectJobSucceeded(page: Page, timeout = 600_000) {
  const statusDot = jobDetailsBox(page).locator(".size-4.rounded-full");
  await expect(statusDot).toHaveClass(/bg-success|bg-destructive|bg-warning/, {
    timeout,
  });
  await expect(statusDot).toHaveClass(/bg-success/);
}
