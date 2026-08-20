import { expect, type Locator, type Page } from "@playwright/test";

function jobDetailsBox(page: Page): Locator {
  return page
    .locator("div.rounded-md.border")
    .filter({ hasText: "Job Details" });
}

function escapeRegExp(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

export function jobDetailsArtifact(page: Page, fileName: string): Locator {
  return jobDetailsBox(page)
    .getByText(new RegExp(escapeRegExp(fileName)))
    .first();
}

export async function expectJobSucceeded(page: Page, timeout = 600_000) {
  const statusDot = jobDetailsBox(page).locator(".size-4.rounded-full");
  await expect(statusDot).toHaveClass(/bg-success|bg-destructive|bg-warning/, {
    timeout,
  });
  await expect(statusDot).toHaveClass(/bg-success/);
}
