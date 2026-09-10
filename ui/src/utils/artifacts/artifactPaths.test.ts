import { describe, expect, test } from "vitest";

import {
  folderArchiveName,
  groupArtifactsByFolder,
  toArtifactFiles,
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
    expect(folderArchiveName("job_abc", "out.geojson")).toBe(
      "job_abc_out.geojson.zip",
    );
  });

  // A `/` cannot appear in a file name.
  test("flattens a nested folder path", () => {
    expect(folderArchiveName("job_abc", "a/b/c")).toBe("job_abc_a_b_c.zip");
  });
});
