import { describeGeometry, isNextFormat } from "@flow/lib/intermediateData";

type GeometryType = string | null;

/**
 * Readable names for the legacy geometry types, so a legacy file and a
 * new-format one — which reads its names from the engine's schema — do not
 * label the same header in two different styles.
 */
const LEGACY_TYPE_LABELS: Record<string, string> = {
  FlowGeometry2D: "2D geometry",
  FlowGeometry3D: "3D geometry",
  CityGmlGeometry: "CityGML geometry",
};

function detectGeometryType(feature: any): GeometryType {
  // New format: the geometry's own key is its type, so there is nothing to
  // infer — read the label the engine's schema gives it.
  if (isNextFormat(feature)) {
    const described = describeGeometry(feature.geometry);
    if (described.kind === "none") return null;
    return described.label || described.variant || "Unknown";
  }

  const geometryValue = feature?.geometry?.value;

  if (!geometryValue) return null;

  // Check for FlowGeometry2D (both casing variations)
  if (geometryValue.FlowGeometry2D || geometryValue.flowGeometry2D)
    return "FlowGeometry2D";

  // Check for FlowGeometry3D (both casing variations)
  if (geometryValue.FlowGeometry3D || geometryValue.flowGeometry3D)
    return "FlowGeometry3D";

  // Check for CityGmlGeometry (multiple casing variations)
  if (geometryValue.CityGmlGeometry || geometryValue.cityGmlGeometry)
    return "CityGmlGeometry";

  return "Unknown";
}

/** Shown when a file holds more than one kind of geometry. */
const MIXED_LABEL = "Mixed";

/**
 * The one type a file holds, or {@link MIXED_LABEL} when it holds several.
 *
 * Naming the most common would present a mixed file as uniform — a file of
 * polylines, points and polygons is not a file of polylines. The per-row
 * `geometry.type` column carries the detail; this is only the headline.
 */
function singleType(labels: string[]): string | null {
  const distinct = new Set(labels);
  if (distinct.size === 0) return null;
  if (distinct.size === 1) return [...distinct][0];
  return MIXED_LABEL;
}

/** The label for new-format data: its type, when the sample agrees on one. */
function analyzeNextFormat(sample: any[]): { geometryType: GeometryType } {
  const labels = sample
    .map((feature) => describeGeometry(feature?.geometry))
    .filter((entry) => entry.kind !== "none" && entry.kind !== "unknown")
    .map((entry) => entry.label || entry.variant || "Unknown");

  return { geometryType: singleType(labels) };
}

/**
 * The geometry label for a file, from a sample of its raw features.
 *
 * Exported for testing: it is the one place both geometry formats have to
 * agree.
 */
export function analyzeDataType(features: any[]): {
  geometryType: GeometryType;
} {
  if (features.length === 0) return { geometryType: null };

  // Check first few features to determine predominant type
  const sampleSize = Math.min(10, features.length);
  const sample = features.slice(0, sampleSize);

  if (sample.some(isNextFormat)) return analyzeNextFormat(sample);

  const typeCounts: Record<string, number> = {};

  for (let i = 0; i < sampleSize; i++) {
    const type = detectGeometryType(features[i]);
    if (type && type !== "Unknown") {
      typeCounts[type] = (typeCounts[type] || 0) + 1;
    }
  }

  // Return most common type, or null if no geometry types found
  const entries = Object.entries(typeCounts);
  if (entries.length === 0) return { geometryType: null };

  // If we have mixed types or mostly unknown, return null instead of confusing info
  const totalGeometryFeatures = Object.values(typeCounts).reduce(
    (sum, count) => sum + count,
    0,
  );
  if (totalGeometryFeatures < sampleSize / 2) {
    return { geometryType: null }; // Less than half have recognizable geometry
  }

  // Name the type only when the sample agrees on it.
  const named = entries.map(([type]) => LEGACY_TYPE_LABELS[type] ?? type);

  return { geometryType: singleType(named) };
}
