import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";

import type { Basemap } from "@flow/components/visualizations/TilesViewer/sources";
import { PAPER } from "@flow/components/visualizations/TilesViewer/sources";
import type * as IntermediateDataViewModule from "@flow/lib/gql/intermediateDataView";
import type { IntermediateDataViewState } from "@flow/lib/gql/intermediateDataView/useApi";
import type {
  IntermediateDataView,
  IntermediateDataViewLayer,
} from "@flow/types";

import TilesView from "./index";

let state: IntermediateDataViewState = { isRendering: false };
let basemaps: Basemap[] = [PAPER];

vi.mock("@flow/lib/gql/intermediateDataView", async (importOriginal) => ({
  ...(await importOriginal<typeof IntermediateDataViewModule>()),
  useIntermediateDataView: () => ({
    useView: () => state,
    requestView: vi.fn(),
    errorMessage: () => "message",
  }),
}));

vi.mock("@flow/components/visualizations/TilesViewer/useBasemaps", () => ({
  useBasemaps: () => basemaps,
}));

// The map engine needs WebGL and WebAssembly, so a stand-in shows what it was
// handed and lets a test call it back.
vi.mock("@flow/components/visualizations/TilesViewer", () => ({
  default: (props: {
    layers: { format: string; entryPointUrl: string }[];
    terrain?: boolean;
    basemap?: string;
    onHeights?: (heights: { min: number; max: number }) => void;
    onError?: (error: unknown) => void;
  }) => (
    <div>
      <span>
        {props.layers
          .map(
            ({ format, entryPointUrl }) => `${format} map of ${entryPointUrl}`,
          )
          .join(" and ")}
        {props.terrain ? " with terrain" : ""} on {props.basemap}
      </span>
      <button onClick={() => props.onHeights?.({ min: 51.34, max: 85.06 })}>
        report heights
      </button>
      <button onClick={() => props.onError?.(new Error("no tileset"))}>
        fail
      </button>
    </div>
  ),
}));

const request = { jobId: "job1", fileId: "n1.default" };

const tiles3d: IntermediateDataViewLayer = {
  format: "cesium3dTiles",
  entryPointUrl: "https://api/v/tileset.json",
  selectedFeatures: 8,
  renderedFeatures: 8,
};

const vectorTiles: IntermediateDataViewLayer = {
  format: "vectorTiles",
  entryPointUrl: "https://api/v/tilejson.json",
  selectedFeatures: 8,
  renderedFeatures: 8,
  sizeLimitedFeatures: 0,
  sizeLimitedTiles: 0,
};

const ready = (
  overrides: Partial<IntermediateDataView> = {},
): IntermediateDataViewState => ({
  isRendering: false,
  view: {
    id: "tiles-abc",
    jobId: "job1",
    fileId: "n1.default",
    shape: "tiles",
    status: "ready",
    layers: [tiles3d],
    selectedFeatures: 8,
    renderedFeatures: 8,
    sizeLimitedFeatures: 0,
    ...overrides,
  },
});

const setup = (overlay?: () => React.ReactNode) =>
  render(
    <TilesView
      request={request}
      overlay={overlay}
      onPickRow={vi.fn()}
      onRetry={vi.fn()}
      onClose={vi.fn()}
    />,
  );

beforeEach(() => {
  state = { isRendering: false };
  basemaps = [PAPER];
});

describe("TilesView", () => {
  test("shows a running render and how long it has taken", () => {
    state = { isRendering: true, renderStartedAt: Date.now() - 5_000 };
    setup();

    expect(screen.getByText(/Rendering the map/)).toBeInTheDocument();
    expect(screen.getByText("0:05")).toBeInTheDocument();
  });

  test("draws a ready view on the default basemap, with its counts and credits", async () => {
    state = ready({ sizeLimitedFeatures: 3 });
    setup();

    expect(
      await screen.findByText(
        "cesium3dTiles map of https://api/v/tileset.json on paper",
      ),
    ).toBeInTheDocument();
    expect(screen.getByText("8 of 8 features drawn")).toBeInTheDocument();
    // One layer needs no counts of its own.
    expect(screen.queryByText(/^3D:/)).not.toBeInTheDocument();
    expect(
      screen.getByText(
        "3 features are left out where tiles reached their size limit.",
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Re:Earth Papers" }),
    ).toHaveAttribute("href", "https://papers.reearth.land/attribution");
    expect(
      screen.getByRole("link", { name: "OpenStreetMap" }),
    ).toBeInTheDocument();
  });

  test("gives the heights the content spans", async () => {
    state = ready();
    setup();

    fireEvent.click(await screen.findByText("report heights"));
    expect(screen.getByText("Heights 51.3 to 85.1 m")).toBeInTheDocument();
  });

  test("offers terrain for 3D Tiles, and credits it while it is shown", async () => {
    state = ready();
    setup();
    await screen.findByText(/cesium3dTiles map/);

    expect(
      screen.queryByRole("link", { name: "Re:Earth Terrain" }),
    ).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("switch", { name: "Terrain" }));

    expect(screen.getByText(/with terrain/)).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Re:Earth Terrain" }),
    ).toBeInTheDocument();
  });

  test("offers terrain for vector tiles too, which lie on it", async () => {
    state = ready({ layers: [vectorTiles] });
    setup();
    await screen.findByText(/vectorTiles map/);

    fireEvent.click(screen.getByRole("switch", { name: "Terrain" }));
    expect(screen.getByText(/with terrain/)).toBeInTheDocument();
  });

  test("draws a port's 3D and 2D layers on one map, with each one's counts", async () => {
    state = ready({
      layers: [tiles3d, { ...vectorTiles, renderedFeatures: 7 }],
      selectedFeatures: 16,
      renderedFeatures: 15,
    });
    setup();

    expect(
      await screen.findByText(
        "cesium3dTiles map of https://api/v/tileset.json and vectorTiles map of https://api/v/tilejson.json on paper",
      ),
    ).toBeInTheDocument();
    expect(screen.getByText("15 of 16 features drawn")).toBeInTheDocument();
    expect(screen.getByText("3D: 8 of 8")).toBeInTheDocument();
    expect(screen.getByText("2D: 7 of 8")).toBeInTheDocument();
  });

  test("offers a choice of basemap only when there is one to make", async () => {
    state = ready();
    setup();
    await screen.findByText(/cesium3dTiles map/);
    expect(
      screen.queryByRole("combobox", { name: "Basemap" }),
    ).not.toBeInTheDocument();
  });

  test("says when the map cannot start, and starts it again on request", async () => {
    state = ready();
    setup();

    fireEvent.click(await screen.findByText("fail"));
    expect(
      screen.getByText("The map could not be displayed."),
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(await screen.findByText(/cesium3dTiles map/)).toBeInTheDocument();
  });

  test("puts the overlay over the map", async () => {
    state = ready();
    setup(() => <span>the selected row</span>);

    expect(await screen.findByText("the selected row")).toBeInTheDocument();
  });
});
