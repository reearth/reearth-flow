import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";

import type { GeometrySummary } from "@flow/lib/intermediateData";
import type {
  IntermediateDataViewFormat,
  IntermediateDataViewLayer,
} from "@flow/types";

import RowCard from "./RowCard";

const summary = (
  overrides: Partial<GeometrySummary> = {},
): GeometrySummary => ({
  label: "Geometry collection",
  has2D: false,
  has3D: true,
  crs: [6697],
  hasPartWithoutCrs: false,
  has3DWithCrs: true,
  has3DSurface: true,
  has3DNonSurface: false,
  ...overrides,
});

const layer = (
  format: IntermediateDataViewFormat,
): IntermediateDataViewLayer => ({
  format,
  entryPointUrl: `https://api/v/${format}`,
  selectedFeatures: 8,
  renderedFeatures: 8,
});

// A port holding both 3D and 2D geometry.
const mixed = [layer("cesium3dTiles"), layer("vectorTiles")];

const row = (
  geometrySummary: GeometrySummary,
  geometry: unknown = {
    type: "Polygon",
    coordinates: [
      [
        [139.7, 35.6, 52.1],
        [139.8, 35.6, 71.4],
        [139.8, 35.7, 60],
      ],
    ],
  },
) => ({
  id: JSON.stringify("bldg-1"),
  _row: 2,
  _values: { geometry, attributes: { name: "Station" }, geometrySummary },
});

const noop = () => {};

describe("RowCard", () => {
  test("says how the feature sits on the map, not what it says", () => {
    render(
      <RowCard
        feature={row(summary())}
        layers={[layer("cesium3dTiles")]}
        onShowDetails={noop}
      />,
    );

    expect(
      screen.getByText("Geometry collection · 3D · EPSG:6697"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Heights 52.1 to 71.4 m, as recorded"),
    ).toBeInTheDocument();
    // Its attributes are in the table beside the map.
    expect(screen.queryByText("Station")).not.toBeInTheDocument();
  });

  test("warns when 3D Tiles leave a 3D point out", () => {
    render(
      <RowCard
        feature={row(summary({ has3DSurface: false, has3DNonSurface: true }), {
          type: "Point",
          coordinates: [139.7, 35.6, 12],
        })}
        layers={mixed}
        onShowDetails={noop}
      />,
    );

    expect(
      screen.getByText("Not drawn on this map: 3D Tiles show only surfaces."),
    ).toBeInTheDocument();
  });

  test("names the 3D points and lines left out of a row it otherwise draws", () => {
    render(
      <RowCard
        feature={row(summary({ has2D: true, has3DNonSurface: true }))}
        layers={mixed}
        onShowDetails={noop}
      />,
    );

    expect(
      screen.getByText(
        "Its 3D points and lines are not drawn: 3D Tiles show only surfaces.",
      ),
    ).toBeInTheDocument();
  });

  test("2D points and lines are vector tiles, so there is nothing to warn of", () => {
    const flat = summary({
      has2D: true,
      has3D: false,
      has3DWithCrs: false,
      has3DSurface: false,
    });
    for (const layers of [mixed, [layer("vectorTiles")]]) {
      const { unmount } = render(
        <RowCard
          feature={row(flat, { type: "Point", coordinates: [139.7, 35.6] })}
          layers={layers}
          onShowDetails={noop}
        />,
      );
      expect(screen.queryByText(/3D Tiles/)).not.toBeInTheDocument();
      unmount();
    }
  });

  test("collapses to its title, and opens again", () => {
    render(<RowCard feature={row(summary())} onShowDetails={noop} />);

    fireEvent.click(screen.getByRole("button", { name: "Collapse" }));
    expect(screen.queryByText("Details")).not.toBeInTheDocument();
    expect(screen.getByText(/Feature ID:/)).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Expand" }));
    expect(screen.getByText("Details")).toBeInTheDocument();
  });

  test("leads to the full details", () => {
    const onShowDetails = vi.fn();
    render(<RowCard feature={row(summary())} onShowDetails={onShowDetails} />);

    fireEvent.click(screen.getByText("Details"));
    expect(onShowDetails).toHaveBeenCalled();
    // Only offered when the row can be a model.
    expect(screen.queryByText("Open in 3D")).not.toBeInTheDocument();
  });
});
