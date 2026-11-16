import { describe, expect, test } from "vitest";

import { cmsItemPath, cmsItemUrl } from "./cmsItem";

const item = {
  projectId: "p1",
  modelId: "m1",
  itemId: "i1",
  connectionId: "c1",
  projectName: "Wayfarer",
  modelName: "Buildings",
  itemTitle: "Shinjuku 3-chome",
};

describe("cmsItemPath", () => {
  test("names the project, model and item", () => {
    expect(cmsItemPath(item)).toBe("Wayfarer › Buildings › Shinjuku 3-chome");
  });
});

describe("cmsItemUrl", () => {
  test("links to the item's page in CMS", () => {
    expect(cmsItemUrl("https://cms.example.com", "ws1", item)).toBe(
      "https://cms.example.com/workspace/ws1/project/p1/content/m1/details/i1",
    );
  });

  test("ignores a trailing slash on the CMS URL", () => {
    expect(cmsItemUrl("https://cms.example.com/", "ws1", item)).toBe(
      "https://cms.example.com/workspace/ws1/project/p1/content/m1/details/i1",
    );
  });

  test("has no link when no CMS URL is configured", () => {
    expect(cmsItemUrl(undefined, "ws1", item)).toBeUndefined();
    expect(cmsItemUrl("", "ws1", item)).toBeUndefined();
  });

  test("escapes IDs so they stay in their path segment", () => {
    expect(
      cmsItemUrl("https://cms.example.com", "ws1", { ...item, itemId: "a/b" }),
    ).toBe(
      "https://cms.example.com/workspace/ws1/project/p1/content/m1/details/a%2Fb",
    );
  });
});
