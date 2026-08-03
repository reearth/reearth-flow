/**
 * Reading the engine's intermediate-data shape through its own schema.
 *
 * A feature's geometry is an externally-tagged enum, so the JSON key *is* the
 * type — no sniffing the payload to guess what a geometry is. The generated
 * maps turn those keys into the names the engine intends a UI to show.
 */
import {
  CONTAINS_RASTER,
  DEFINITION_TITLES,
  ENUMS,
  PROPERTY_TITLES,
} from "./schema";

/** Top-level geometry enum, as the schema names it. */
const GEOMETRY = "Geometry";

export type GeometryKind = "none" | "2d" | "3d" | "collection" | "unknown";

export type GeometryDescription = {
  kind: GeometryKind;
  /** Discriminant key of the leaf, e.g. "Point"; null for None/unknown. */
  variant: string | null;
  /** Schema definition the leaf resolves to, e.g. "Point2D". */
  definition: string | null;
  /** Display name, e.g. "Point (2D)". */
  label: string;
  /** The leaf's payload, unwrapped from its discriminant. */
  value: unknown;
};

/**
 * The single key of an externally-tagged enum object, or null when `value`
 * isn't one. Serde writes unit variants as a bare string instead.
 */
function tagOf(value: unknown): string | null {
  if (!value || typeof value !== "object" || Array.isArray(value)) return null;
  const keys = Object.keys(value as Record<string, unknown>);
  return keys.length === 1 ? keys[0] : null;
}

/** Resolve one step of a tagged enum: its variant key and target definition. */
function stepInto(
  enumName: string,
  value: unknown,
): { variant: string; definition: string | null; value: unknown } | null {
  const schema = ENUMS[enumName];
  if (!schema) return null;

  if (typeof value === "string" && schema.units.includes(value)) {
    return { variant: value, definition: null, value: null };
  }

  const tag = tagOf(value);
  if (tag === null || !(tag in schema.variants)) return null;

  return {
    variant: tag,
    definition: schema.variants[tag],
    value: (value as Record<string, unknown>)[tag],
  };
}

/** Display name for a schema definition, falling back to the raw name. */
export function definitionLabel(definition: string | null): string {
  if (!definition) return "";
  return DEFINITION_TITLES[definition] ?? definition;
}

/** Display name for a property, falling back to the raw key. */
export function propertyLabel(
  definition: string | null,
  property: string,
): string {
  if (!definition) return property;
  return PROPERTY_TITLES[definition]?.[property] ?? property;
}

/** Whether a definition can transitively hold encoded image bytes. */
export function canContainRaster(definition: string | null): boolean {
  return definition ? CONTAINS_RASTER.has(definition) : false;
}

/**
 * Classify a new-format `geometry` value by reading its discriminant keys.
 *
 * Returns `kind: "unknown"` for anything that doesn't resolve — which is also
 * what a legacy-format geometry produces, since it is an object with `epsg`
 * and `value` rather than a single tag.
 */
export function describeGeometry(geometry: unknown): GeometryDescription {
  const unknown: GeometryDescription = {
    kind: "unknown",
    variant: null,
    definition: null,
    label: "",
    value: geometry,
  };

  const top = stepInto(GEOMETRY, geometry);
  if (!top) return unknown;

  if (top.variant === "None") {
    return {
      kind: "none",
      variant: "None",
      definition: null,
      label: "None",
      value: null,
    };
  }

  if (top.variant === "GeometryCollection") {
    return {
      kind: "collection",
      variant: top.variant,
      definition: top.definition,
      label: definitionLabel(top.definition),
      value: top.value,
    };
  }

  const kind: GeometryKind = top.variant === "Euclidean2D" ? "2d" : "3d";
  const leaf = top.definition ? stepInto(top.definition, top.value) : null;
  if (!leaf) {
    return { ...unknown, kind, value: top.value };
  }

  return {
    kind,
    variant: leaf.variant,
    definition: leaf.definition,
    label: definitionLabel(leaf.definition),
    value: leaf.value,
  };
}

/**
 * Whether a parsed JSONL line uses the new geometry model. The legacy form
 * wraps its payload in `{ epsg, value }`; the new one is the tagged enum
 * itself, and `"None"` as a bare string.
 */
export function isNextFormat(feature: unknown): boolean {
  const geometry = (feature as { geometry?: unknown })?.geometry;
  if (geometry === undefined) return false;
  if (geometry === "None") return true;

  const tag = tagOf(geometry);
  return (
    tag !== null &&
    (tag === "Euclidean2D" ||
      tag === "Euclidean3D" ||
      tag === "GeometryCollection")
  );
}
