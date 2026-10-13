import type { FlyTarget } from "@flow/components/visualizations/TilesViewer";
import { geometryBounds, isGeographicCrs } from "@flow/lib/intermediateData";
import type { GeometrySummary } from "@flow/lib/intermediateData";

/**
 * Where to fly to see a table row on the map, or undefined when its
 * coordinates cannot place it there: only a geographic CRS's are longitude
 * and latitude already.
 */
export const flyTargetOf = (row: any): FlyTarget | undefined => {
  const summary: GeometrySummary | undefined = row?._values?.geometrySummary;
  if (!summary?.crs.length || !summary.crs.every(isGeographicCrs)) return;
  const bounds = geometryBounds(row._values.geometry);
  if (!bounds) return;
  const { heights, ...area } = bounds;
  return { ...area, minHeight: heights?.min, maxHeight: heights?.max };
};
