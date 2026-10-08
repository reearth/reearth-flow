import { describe, expect, test } from "vitest";

import {
  fileIdFromIntermediateDataUrl,
  intermediateDataFileId,
} from "./fileId";

describe("intermediateDataFileId", () => {
  test("joins node and port", () => {
    expect(intermediateDataFileId("n1", "default")).toBe("n1.default");
  });

  test("prefixes the subworkflow path", () => {
    expect(intermediateDataFileId("n1", "default", "sub.inner")).toBe(
      "sub.inner.n1.default",
    );
  });
});

describe("fileIdFromIntermediateDataUrl", () => {
  const base = "https://api.example.com/artifacts/job1/feature-store";

  test("reads the id out of a compressed file's URL", () => {
    expect(fileIdFromIntermediateDataUrl(`${base}/n1.default.jsonl.zst`)).toBe(
      "n1.default",
    );
  });

  test("reads the id out of an uncompressed file's URL", () => {
    expect(fileIdFromIntermediateDataUrl(`${base}/n1.default.jsonl`)).toBe(
      "n1.default",
    );
  });

  test("keeps a subworkflow prefix", () => {
    expect(
      fileIdFromIntermediateDataUrl(`${base}/sub.n1.unfiltered.jsonl.zst`),
    ).toBe("sub.n1.unfiltered");
  });

  test("ignores a query string", () => {
    expect(
      fileIdFromIntermediateDataUrl(`${base}/n1.default.jsonl.zst?token=x`),
    ).toBe("n1.default");
  });

  test("decodes a port name the URL encoded", () => {
    // Port names are free strings, so they can hold spaces.
    expect(
      fileIdFromIntermediateDataUrl(`${base}/n1.not%20matched.jsonl.zst`),
    ).toBe("n1.not matched");
  });

  test("round-trips an id built for the same port", () => {
    const fileId = intermediateDataFileId("n1", "default", "sub");
    expect(fileIdFromIntermediateDataUrl(`${base}/${fileId}.jsonl.zst`)).toBe(
      fileId,
    );
  });

  test("is undefined for a URL that is not a feature-store file", () => {
    expect(
      fileIdFromIntermediateDataUrl(
        "https://api.example.com/artifacts/job1/output.csv",
      ),
    ).toBeUndefined();
    expect(
      fileIdFromIntermediateDataUrl(`${base}/n1.default.csv`),
    ).toBeUndefined();
  });
});
