/**
 * Finding the images inside a parsed feature, by following the schema rather
 * than guessing at key names.
 *
 * Appearances hang off the surface leaves, and a leaf can sit behind
 * collections, a solid's shells, or CSG operands — so the paths are many and
 * none of them are shallow. The generated maps describe those edges, and
 * `CONTAINS_RASTER` says which definitions can reach an image at all, so the
 * walk prunes at the first node that cannot: coordinate rings, point-cloud
 * segments and UV sets, which is where nearly all of a feature's bulk lives.
 *
 * The walk stops at `Appearance` and reads its materials directly. Every path
 * to a texture runs through one — `Raster` is reachable only from `Texture`,
 * `Texture` only from a material, and a material only from
 * `Appearance.materials` — so nothing is missed, and the material layer is
 * small and fixed enough to read plainly.
 */
import { canContainRaster, definitionLabel, propertyLabel } from "./labels";
import { internRaster, type RasterHandle } from "./rasterStore";
import { ENUMS, PROPERTY_TARGETS } from "./schema";

/** Definition names the walk needs to recognise by hand. */
const GEOMETRY = "Geometry";
const APPEARANCE = "Appearance";
const TEXTURE = "Texture";
const IN_MEMORY = "InMemory";

/** One texture map on a material, e.g. a PBR base colour map. */
export type TextureSlot = {
  /** Property key on the material, e.g. "base_color_map". */
  slot: string;
  /** The schema's name for it, e.g. "Base color map". */
  label: string;
  /** Present when the image travelled inside the feature. */
  image?: RasterHandle;
  /** Present when the image is named by location instead. */
  uri?: string;
};

export type MaterialSummary = {
  /** Shading model: "Phong" or "Pbr". */
  kind: string;
  label: string;
  textures: TextureSlot[];
};

export type AppearanceSummary = {
  materials: MaterialSummary[];
  /** Every embedded image found, flattened. */
  textures: RasterHandle[];
};

/** Sole key of an externally-tagged enum object, or null. */
function tagOf(value: unknown): string | null {
  if (!value || typeof value !== "object" || Array.isArray(value)) return null;
  const keys = Object.keys(value as Record<string, unknown>);
  return keys.length === 1 ? keys[0] : null;
}

/** The texture-valued properties of a material, per the schema. */
function textureSlotsOf(materialDefinition: string): string[] {
  const properties = PROPERTY_TARGETS[materialDefinition] ?? {};
  return Object.entries(properties)
    .filter(([, target]) => target === TEXTURE)
    .map(([property]) => property);
}

/**
 * Read one texture: lift any embedded bytes into the store, and note an
 * external location as-is. `Raster::Uri` names a file and carries no pixels.
 */
function readTexture(
  texture: unknown,
  owner: string,
  found: RasterHandle[],
): Pick<TextureSlot, "image" | "uri"> | null {
  const raster = (texture as Record<string, unknown>)?.raster;
  if (!raster || typeof raster !== "object") return null;

  const record = raster as Record<string, unknown>;

  if (typeof record.Uri === "string") return { uri: record.Uri };

  const payload = record[IN_MEMORY];
  if (!payload || typeof payload !== "object") return null;

  const data = payload as Record<string, unknown>;
  const handle = internRaster(owner, data.mime_type, data.bytes);
  if (!handle) return null;

  // Replace the bytes in place; the caller owns this freshly parsed object.
  record[IN_MEMORY] = handle;
  found.push(handle);
  return { image: handle };
}

function readAppearance(
  appearance: unknown,
  owner: string,
  materials: MaterialSummary[],
  found: RasterHandle[],
): void {
  const palette = (appearance as Record<string, unknown>)?.materials;
  if (!Array.isArray(palette)) return;

  for (const material of palette) {
    const kind = tagOf(material);
    if (!kind) continue;

    const definition = ENUMS.Material?.variants[kind];
    if (!definition) continue;

    const body = (material as Record<string, unknown>)[kind] as
      | Record<string, unknown>
      | undefined;
    if (!body) continue;

    const textures: TextureSlot[] = [];
    for (const slot of textureSlotsOf(definition)) {
      const read = readTexture(body[slot], owner, found);
      if (read) {
        textures.push({
          slot,
          label: propertyLabel(definition, slot),
          ...read,
        });
      }
    }

    materials.push({
      kind,
      label: definitionLabel(definition),
      textures,
    });
  }
}

function walk(
  node: unknown,
  definition: string,
  owner: string,
  materials: MaterialSummary[],
  found: RasterHandle[],
): void {
  if (node === null || node === undefined) return;
  if (!canContainRaster(definition)) return;

  // One definition covers both `Vec<T>` and the fixed tuples `Csg` uses.
  if (Array.isArray(node)) {
    for (const item of node) walk(item, definition, owner, materials, found);
    return;
  }
  if (typeof node !== "object") return;

  if (definition === APPEARANCE) {
    readAppearance(node, owner, materials, found);
    return;
  }

  const enumSchema = ENUMS[definition];
  if (enumSchema) {
    const tag = tagOf(node);
    if (tag === null) return;
    const target = enumSchema.variants[tag];
    if (target) {
      walk(
        (node as Record<string, unknown>)[tag],
        target,
        owner,
        materials,
        found,
      );
    }
    return;
  }

  const properties = PROPERTY_TARGETS[definition];
  if (!properties) return;
  for (const [property, target] of Object.entries(properties)) {
    const value = (node as Record<string, unknown>)[property];
    if (value !== undefined) walk(value, target, owner, materials, found);
  }
}

/**
 * Read a feature's appearance, lifting every embedded image out of it.
 *
 * The geometry is mutated in place: it has just come out of `JSON.parse` and
 * is owned by the caller, and copying it would mean copying the byte arrays
 * this exists to get rid of.
 */
export function extractAppearance(
  geometry: unknown,
  owner: string,
): AppearanceSummary {
  const materials: MaterialSummary[] = [];
  const textures: RasterHandle[] = [];
  walk(geometry, GEOMETRY, owner, materials, textures);
  return { materials, textures };
}
