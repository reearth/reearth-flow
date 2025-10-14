/**
 * Utilities for detecting and working with temporal properties in GeoJSON data
 */

export type TimelineProperty = {
  name: string;
  values: (string | number)[];
  min: string | number;
  max: string | number;
};

/**
 * Check if a value looks temporal (year, date, timestamp)
 */
function isTemporalValue(value: unknown): boolean {
  if (value === null || value === undefined) return false;

  // Check for year (number between 1900-2100)
  if (typeof value === "number" && value >= 1900 && value <= 2100) {
    return true;
  }

  // Check for timestamp (large number)
  if (typeof value === "number" && value > 946684800000) {
    // After Jan 1, 2000
    return true;
  }

  // Check for date string patterns
  if (typeof value === "string") {
    const datePatterns = [
      /^\d{4}$/, // YYYY
      /^\d{4}-\d{2}$/, // YYYY-MM
      /^\d{4}-\d{2}-\d{2}$/, // YYYY-MM-DD
      /^\d{4}\/\d{2}\/\d{2}$/, // YYYY/MM/DD
      /^\d{2}\/\d{2}\/\d{4}$/, // MM/DD/YYYY
      /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}/, // ISO 8601 with time: 2025-01-01T00:00:00
      /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z?/, // ISO 8601 with milliseconds: 2025-01-01T00:00:00.000Z
      /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}[+-]\d{2}:\d{2}/, // ISO 8601 with timezone: 2025-01-01T00:00:00+00:00
    ];

    // First check with patterns for performance
    if (datePatterns.some((pattern) => pattern.test(value))) {
      return true;
    }

    // Fallback: try parsing as date if it looks date-like
    if (value.includes("-") || value.includes("/") || value.includes("T")) {
      const date = new Date(value);
      return !isNaN(date.getTime());
    }
  }

  return false;
}

/**
 * Convert temporal value to comparable number for sorting/filtering
 */
export function toComparableValue(value: string | number): number {
  if (typeof value === "number") return value;

  // Try parsing as year
  const year = parseInt(value, 10);
  if (!isNaN(year) && year >= 1900 && year <= 2100) return year;

  // Try parsing as date
  const date = new Date(value);
  if (!isNaN(date.getTime())) return date.getTime();

  return 0;
}

/**
 * Detect temporal properties in GeoJSON data
 */
export function detectTemporalProperties(
  geojson: GeoJSON.FeatureCollection | null,
): TimelineProperty[] {
  if (!geojson?.features || geojson.features.length === 0) return [];

  const propertyMap = new Map<string, Set<string | number>>();

  // Scan all features to find temporal properties
  geojson.features.forEach((feature) => {
    if (!feature.properties) return;

    Object.entries(feature.properties).forEach(([key, value]) => {
      if (!isTemporalValue(value)) return;

      if (!propertyMap.has(key)) {
        propertyMap.set(key, new Set());
      }
      propertyMap.get(key)?.add(value as string | number);
    });
  });

  // Convert to TimelineProperty array
  const properties: TimelineProperty[] = [];

  propertyMap.forEach((valueSet, name) => {
    // Skip if less than 2 unique values
    if (valueSet.size < 2) return;

    const values = Array.from(valueSet).sort((a, b) => {
      return toComparableValue(a) - toComparableValue(b);
    });

    properties.push({
      name,
      values,
      min: values[0],
      max: values[values.length - 1],
    });
  });

  return properties;
}

/**
 * Filter GeoJSON features by temporal property and value
 */
export function filterByTimelineValue(
  geojson: GeoJSON.FeatureCollection | null,
  propertyName: string,
  value: string | number,
): GeoJSON.FeatureCollection | null {
  if (!geojson) return null;

  const targetValue = toComparableValue(value);

  return {
    ...geojson,
    features: geojson.features.filter((feature) => {
      if (!feature.properties) return false;

      const featureValue = feature.properties[propertyName];
      if (!featureValue) return false;

      const comparable = toComparableValue(featureValue);
      return comparable <= targetValue; // Show features up to selected time
    }),
  };
}

/**
 * Format temporal value for display
 */
export function formatTimelineValue(value: string | number): string {
  if (typeof value === "number") {
    // Year range
    if (value >= 1900 && value <= 2100) return String(value);

    // Timestamp
    return new Date(value).toLocaleDateString();
  }

  // Try to format date string
  const date = new Date(value);
  if (!isNaN(date.getTime())) {
    return date.toLocaleDateString();
  }

  return String(value);
}
