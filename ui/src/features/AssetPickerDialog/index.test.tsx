import { fireEvent, render, screen } from "@testing-library/react";
import { beforeAll, describe, expect, it, vi } from "vitest";

import type { CmsAsset, CmsProject } from "@flow/types/cmsIntegration";

import AssetPickerDialog from ".";

vi.mock("@flow/lib/i18n", () => ({
  useT: () => (key: string) => key,
}));

vi.mock("@flow/stores", () => ({
  useCurrentWorkspace: () => [{ id: "ws-1" }],
}));

vi.mock("@flow/features/AssetsDialog", () => ({
  WorkspaceAssetsPanel: () => <div>workspace assets panel</div>,
}));

vi.mock("./components/CmsModelsPanel", () => ({
  default: () => <div>cms models panel</div>,
}));

const project = (id: string, name: string): CmsProject => ({
  id,
  name,
  alias: id,
  workspaceId: "ws-1",
  visibility: "public",
  createdAt: "2026-01-01T00:00:00Z",
  updatedAt: "2026-01-01T00:00:00Z",
});

const asset = (
  id: string,
  projectId: string,
  filename: string,
  isPublic = true,
): CmsAsset => ({
  id,
  uuid: id,
  projectId,
  filename,
  size: 1024,
  url: `https://cms.example.com/${filename}`,
  public: isPublic,
  createdAt: "2026-01-01T00:00:00Z",
});

const projects = [project("p1", "Tsukuba"), project("p2", "Sensors")];
const assets = [
  asset("a1", "p1", "bldg.gml"),
  asset("a2", "p2", "points.geojson"),
  asset("a3", "p1", "secret.csv", false),
];

const queriedProjectIds = vi.fn();

vi.mock("@flow/lib/gql/cms", () => ({
  useCms: () => ({
    useGetCmsProjects: () => ({
      page: { cmsProjects: projects },
      isFetching: false,
    }),
    useGetCmsAssetsAcrossProjects: (queried: CmsProject[]) => {
      queriedProjectIds(queried.map((p) => p.id));
      return {
        cmsAssets: assets.filter((a) =>
          queried.some((p) => p.id === a.projectId),
        ),
        failedProjectCount: 0,
        hasNextPage: false,
        isFetching: false,
        isFetchingNextPage: false,
        fetchNextPage: vi.fn(),
      };
    },
  }),
}));

beforeAll(() => {
  globalThis.ResizeObserver = class {
    observe() {}
    unobserve() {}
    disconnect() {}
  } as unknown as typeof ResizeObserver;
});

const renderPicker = () => {
  const onSelect = vi.fn();
  const onDialogClose = vi.fn();
  render(
    <AssetPickerDialog onSelect={onSelect} onDialogClose={onDialogClose} />,
  );
  fireEvent.click(screen.getByRole("tab", { name: /CMS Assets/ }));
  return { onSelect, onDialogClose };
};

describe("AssetPickerDialog", () => {
  it("opens on Flow workspace assets", () => {
    render(<AssetPickerDialog onSelect={vi.fn()} onDialogClose={vi.fn()} />);
    expect(screen.getByText("workspace assets panel")).toBeInTheDocument();
  });
});

describe("AssetPickerDialog CMS Assets tab", () => {
  it("lists public assets from every project and hides private ones", () => {
    renderPicker();
    expect(queriedProjectIds).toHaveBeenLastCalledWith(["p1", "p2"]);
    expect(screen.getByText("bldg.gml")).toBeInTheDocument();
    expect(screen.getByText("points.geojson")).toBeInTheDocument();
    expect(screen.queryByText("secret.csv")).not.toBeInTheDocument();
  });

  it("shows which project each asset comes from", () => {
    renderPicker();
    expect(screen.getByText("Tsukuba")).toBeInTheDocument();
    expect(screen.getByText("Sensors")).toBeInTheDocument();
  });

  it("selects the clicked asset's URL with the Select button", () => {
    const { onSelect, onDialogClose } = renderPicker();
    const select = screen.getByRole("button", { name: "Select" });
    expect(select).toBeDisabled();

    fireEvent.click(screen.getByText("points.geojson"));
    expect(select).toBeEnabled();
    fireEvent.click(select);

    expect(onSelect).toHaveBeenCalledWith(
      "https://cms.example.com/points.geojson",
    );
    expect(onDialogClose).toHaveBeenCalled();
  });

  it("selects immediately on double click", () => {
    const { onSelect } = renderPicker();
    fireEvent.doubleClick(screen.getByText("bldg.gml"));
    expect(onSelect).toHaveBeenCalledWith("https://cms.example.com/bldg.gml");
  });

  it("filters by file name", () => {
    renderPicker();
    fireEvent.change(screen.getByPlaceholderText("Search..."), {
      target: { value: "bldg" },
    });
    expect(screen.getByText("bldg.gml")).toBeInTheDocument();
    expect(screen.queryByText("points.geojson")).not.toBeInTheDocument();
  });

  it("does not carry a selection over to another tab", () => {
    renderPicker();
    fireEvent.click(screen.getByText("bldg.gml"));
    fireEvent.click(screen.getByRole("tab", { name: /Flow Workspace Assets/ }));
    expect(screen.getByText("workspace assets panel")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Select" })).toBeDisabled();
  });
});
