import { act, renderHook } from "@testing-library/react";
import JSZip from "jszip";

import { toArtifactFiles } from "@flow/utils";

import useArtifactZipDownload from "./useArtifactZipDownload";

const saveAs = vi.fn();
vi.mock("file-saver", () => ({
  saveAs: (...args: unknown[]) => saveAs(...args),
}));

const root = "https://example.com/artifacts/job-id/artifacts";

const savedArchive = async () => {
  const [blob, name] = saveAs.mock.calls[0] as [Blob, string];
  const zip = await JSZip.loadAsync(await blob.arrayBuffer());
  const files = Object.values(zip.files)
    .filter((entry) => !entry.dir)
    .map((entry) => entry.name)
    .sort();
  return { name, files };
};

describe("useArtifactZipDownload", () => {
  beforeEach(() => {
    saveAs.mockReset();
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string) => new Response(`contents of ${url}`)),
    );
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  test("keeps every writer's folders inside the archive", async () => {
    const files = toArtifactFiles([
      `${root}/udx/bldg/53394525_bldg_6697_op.gml`,
      `${root}/udx/tran/53394525_tran_6697_op.gml`,
      `${root}/codelists/Building_class.xml`,
      `${root}/summary.json`,
    ]);
    const { result } = renderHook(() => useArtifactZipDownload());

    await act(() => result.current.downloadAsZip(files, "Tokyo bldg.zip"));

    expect(await savedArchive()).toEqual({
      name: "Tokyo bldg.zip",
      files: [
        "codelists/Building_class.xml",
        "summary.json",
        "udx/bldg/53394525_bldg_6697_op.gml",
        "udx/tran/53394525_tran_6697_op.gml",
      ],
    });
  });

  // A folder's archive extracts to the same place in the tree as the whole
  // job's would, rather than dropping its files loose.
  test("keeps the folder path for one folder's files", async () => {
    const files = toArtifactFiles([`${root}/udx/bldg/a.gml`]);
    const { result } = renderHook(() => useArtifactZipDownload());

    await act(() => result.current.downloadAsZip(files, "udx_bldg.zip"));

    expect((await savedArchive()).files).toEqual(["udx/bldg/a.gml"]);
  });
});
