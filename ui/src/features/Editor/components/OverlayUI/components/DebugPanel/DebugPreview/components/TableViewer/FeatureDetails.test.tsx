import { render, screen } from "@testing-library/react";
import { describe, expect, test } from "vitest";

import type { GeometrySummary } from "@flow/lib/intermediateData";

import FeatureDetails from "./FeatureDetails";

const summary = (
  overrides: Partial<GeometrySummary> = {},
): GeometrySummary => ({
  label: "Geometry collection",
  has2D: false,
  has3D: true,
  crs: [6697],
  hasPartWithoutCrs: false,
  has3DWithCrs: true,
  ...overrides,
});

const row = (geometrySummary?: GeometrySummary) => ({
  id: JSON.stringify("bldg-1"),
  _row: 2,
  _values: {
    geometry: { type: "MultiPolygon" },
    attributes: { name: "Station" },
    geometrySummary,
  },
});

describe("FeatureDetails as a pane", () => {
  test("says what the geometry is and where it is", () => {
    render(<FeatureDetails variant="pane" feature={row(summary())} />);

    expect(screen.getByText("Geometry collection")).toBeDefined();
    expect(screen.getByText("3D")).toBeDefined();
    expect(screen.getByText("EPSG:6697")).toBeDefined();
  });

  test("warns that geometry with no CRS is left out of rendered views", () => {
    render(
      <FeatureDetails
        variant="pane"
        feature={row(summary({ crs: [], hasPartWithoutCrs: true }))}
      />,
    );

    expect(screen.getByText(/has no CRS/)).toBeDefined();
  });

  test("shows the actions it is given", () => {
    render(
      <FeatureDetails
        variant="pane"
        feature={row(summary())}
        actions={<button type="button">Open in 3D</button>}
      />,
    );

    expect(screen.getByRole("button", { name: "Open in 3D" })).toBeDefined();
  });

  test("leaves the keyboard with the table", () => {
    // The table's arrow keys move the selection the pane shows, so the pane
    // must not take focus when it appears.
    render(<FeatureDetails variant="pane" feature={row(summary())} />);

    expect(document.activeElement).toBe(document.body);
  });

  test("shows no summary for a row without geometry", () => {
    render(<FeatureDetails variant="pane" feature={row(undefined)} />);

    expect(screen.queryByText("EPSG:6697")).toBeNull();
  });
});
