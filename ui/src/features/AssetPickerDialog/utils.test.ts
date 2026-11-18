import { describe, expect, it } from "vitest";

import type { CmsAsset } from "@flow/types/cmsIntegration";

import { cmsAssetFormat, filterAndSortCmsAssets } from "./utils";

const asset = (filename: string, createdAt: string, size = 0): CmsAsset => ({
  id: filename,
  uuid: filename,
  projectId: "p",
  filename,
  size,
  url: `https://cms.example.com/${filename}`,
  public: true,
  createdAt,
});

describe("cmsAssetFormat", () => {
  it.each([
    ["bldg.gml", "citygml"],
    ["points.GeoJSON", "geojson"],
    ["tileset.json", "json"],
    ["udx.zip", "archive"],
    ["model.glb", "model"],
    ["README", "other"],
    ["notes.txt", "other"],
  ])("classifies %s as %s", (filename, format) => {
    expect(cmsAssetFormat(filename)).toBe(format);
  });
});

describe("filterAndSortCmsAssets", () => {
  const assets = [
    asset("b.gml", "2026-01-02T00:00:00Z", 30),
    asset("a.geojson", "2026-03-01T00:00:00Z", 10),
    asset("c.gml", "2026-02-01T00:00:00Z", 20),
  ];

  it("sorts newest first by timestamp, not by display string", () => {
    const sorted = filterAndSortCmsAssets(assets, { sort: "createdAtDesc" });
    expect(sorted.map((a) => a.filename)).toEqual([
      "a.geojson",
      "c.gml",
      "b.gml",
    ]);
  });

  it("orders RFC3339 timestamps with trimmed fractional seconds", () => {
    const sorted = filterAndSortCmsAssets(
      [
        asset("later.gml", "2026-01-01T00:00:00.5Z"),
        asset("earlier.gml", "2026-01-01T00:00:00Z"),
      ],
      { sort: "createdAtAsc" },
    );
    expect(sorted.map((a) => a.filename)).toEqual(["earlier.gml", "later.gml"]);
  });

  it("filters by format and keyword together", () => {
    const filtered = filterAndSortCmsAssets(assets, {
      format: "citygml",
      keyword: "C",
      sort: "nameAsc",
    });
    expect(filtered.map((a) => a.filename)).toEqual(["c.gml"]);
  });

  it("sorts by size", () => {
    const sorted = filterAndSortCmsAssets(assets, { sort: "sizeAsc" });
    expect(sorted.map((a) => a.size)).toEqual([10, 20, 30]);
  });

  it("does not reorder the input", () => {
    const input = [...assets];
    filterAndSortCmsAssets(input, { sort: "nameAsc" });
    expect(input).toEqual(assets);
  });
});
