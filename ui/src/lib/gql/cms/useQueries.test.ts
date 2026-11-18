import { describe, expect, it } from "vitest";

import type { CmsAssetsPage } from "./useQueries";
import { nextCmsAssetsPageParam } from "./useQueries";

const page = (
  loaded: CmsAssetsPage["loaded"],
  failedProjectIds: string[] = [],
): CmsAssetsPage => ({ assets: [], loaded, failedProjectIds });

describe("nextCmsAssetsPageParam", () => {
  it("asks for the next page of every project that has more", () => {
    const next = nextCmsAssetsPageParam(
      [
        page({
          a: { page: 1, totalCount: 120 },
          b: { page: 1, totalCount: 30 },
        }),
      ],
      50,
    );
    expect(next).toEqual({ a: 2 });
  });

  it("uses each project's latest loaded page across all pages", () => {
    const next = nextCmsAssetsPageParam(
      [
        page({
          a: { page: 1, totalCount: 120 },
          b: { page: 1, totalCount: 60 },
        }),
        page({
          a: { page: 2, totalCount: 120 },
          b: { page: 2, totalCount: 60 },
        }),
      ],
      50,
    );
    expect(next).toEqual({ a: 3 });
  });

  it("skips projects that failed and never loaded", () => {
    const next = nextCmsAssetsPageParam(
      [page({ a: { page: 1, totalCount: 10 } }, ["b"])],
      50,
    );
    expect(next).toBeUndefined();
  });

  it("stops when a project's total is exactly a full page", () => {
    expect(
      nextCmsAssetsPageParam([page({ a: { page: 1, totalCount: 50 } })], 50),
    ).toBeUndefined();
  });
});
