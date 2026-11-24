import { describe, expect, test } from "vitest";

import {
  folderArchiveName,
  groupArtifactsByFolder,
  toArchiveBaseName,
  toArtifactFiles,
  toSuggestedArchiveName,
} from "./artifactPaths";

const root = "https://example.com/artifacts/job-id/artifacts";

describe("toArtifactFiles", () => {
  test("keeps the folder a grouped writer created", () => {
    const files = toArtifactFiles([
      `${root}/out.geojson/abc123.geojson`,
      `${root}/out.geojson/def456.geojson`,
    ]);

    expect(files.map((f) => f.path)).toEqual([
      "out.geojson/abc123.geojson",
      "out.geojson/def456.geojson",
    ]);
    expect(files[0].folder).toBe("out.geojson");
    expect(files[0].name).toBe("abc123.geojson");
  });

  test("leaves an ungrouped output at the artifact root", () => {
    const [file] = toArtifactFiles([`${root}/out.geojson`]);

    expect(file.path).toBe("out.geojson");
    expect(file.folder).toBe("");
  });

  test("keeps nesting deeper than one level", () => {
    const [file] = toArtifactFiles([`${root}/a/b/c/out.geojson`]);

    expect(file.path).toBe("a/b/c/out.geojson");
    expect(file.folder).toBe("a/b/c");
  });

  test("falls back to the file name when there is no artifact root", () => {
    const [file] = toArtifactFiles([
      "https://output.reearth.io/job-1/result.json",
    ]);

    expect(file.path).toBe("result.json");
    expect(file.folder).toBe("");
  });

  test("decodes percent-escaped segments", () => {
    const [file] = toArtifactFiles([`${root}/out%20data/my%20file.geojson`]);

    expect(file.path).toBe("out data/my file.geojson");
  });

  test("keeps the original url for downloading", () => {
    const url = `${root}/out.geojson/abc123.geojson`;
    expect(toArtifactFiles([url])[0].url).toBe(url);
  });

  test("handles an empty list", () => {
    expect(toArtifactFiles([])).toEqual([]);
  });
});

describe("groupArtifactsByFolder", () => {
  test("buckets by folder with the root first and names sorted", () => {
    const folders = groupArtifactsByFolder(
      toArtifactFiles([
        `${root}/b.geojson/2.geojson`,
        `${root}/b.geojson/1.geojson`,
        `${root}/a.geojson/1.geojson`,
        `${root}/summary.json`,
      ]),
    );

    expect(folders.map((f) => f.path)).toEqual(["", "a.geojson", "b.geojson"]);
    expect(folders[0].files.map((f) => f.name)).toEqual(["summary.json"]);
    expect(folders[2].files.map((f) => f.name)).toEqual([
      "1.geojson",
      "2.geojson",
    ]);
  });
});

describe("folderArchiveName", () => {
  test("names an archive after the folder it holds", () => {
    expect(folderArchiveName("out.geojson")).toBe("out.geojson");
  });

  // A `/` cannot appear in a file name.
  test("flattens a nested folder path", () => {
    expect(folderArchiveName("udx/bldg")).toBe("udx_bldg");
  });
});

describe("toArchiveBaseName", () => {
  test("keeps an ordinary name, spaces and non-Latin text included", () => {
    expect(toArchiveBaseName("東京 建物 2026")).toBe("東京 建物 2026");
  });

  test("trims surrounding whitespace", () => {
    expect(toArchiveBaseName("  results  ")).toBe("results");
  });

  // The extension is added on save, so a typed one would be doubled.
  test("drops a typed .zip extension", () => {
    expect(toArchiveBaseName("results.ZIP")).toBe("results");
  });

  test("replaces characters a file system refuses", () => {
    expect(toArchiveBaseName('a/b\\c:d*e?f"g<h>i|j')).toBe(
      "a_b_c_d_e_f_g_h_i_j",
    );
  });

  test("drops trailing dots and spaces", () => {
    expect(toArchiveBaseName("results. .")).toBe("results");
  });

  test("leaves nothing for a blank name or only dots", () => {
    expect(toArchiveBaseName("   ")).toBe("");
    expect(toArchiveBaseName(".zip")).toBe("");
    expect(toArchiveBaseName("..")).toBe("");
  });
});

describe("toSuggestedArchiveName", () => {
  test("joins words and drops brackets so the name needs no quoting", () => {
    expect(
      toSuggestedArchiveName(
        "Shapefile to CityGML (PLATEAU bldg LOD1) (import)",
      ),
    ).toBe("Shapefile_to_CityGML_PLATEAU_bldg_LOD1_import");
  });

  test("keeps letters in any script", () => {
    expect(toSuggestedArchiveName("東京 建物　2026")).toBe("東京_建物_2026");
  });

  test("keeps dots, hyphens and underscores inside the name", () => {
    expect(toSuggestedArchiveName("out.geojson")).toBe("out.geojson");
    expect(toSuggestedArchiveName("job_1a2b-3c")).toBe("job_1a2b-3c");
  });

  test("does not start or end with a separator", () => {
    expect(toSuggestedArchiveName("(draft) results!")).toBe("draft_results");
  });

  test("leaves nothing when no letter or digit is left", () => {
    expect(toSuggestedArchiveName("(!)")).toBe("");
  });
});
