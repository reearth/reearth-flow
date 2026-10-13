// A temporary bridge. The map engine does not yet follow 3D Tiles 1.1
// implicit tiling, which is what the server writes, so the tileset is
// rewritten here into the explicit form, with every tile listed. Remove once
// the engine supports implicit tiling.

type Availability = { constant?: number; bitstream?: number };

type Subtree = {
  tileAvailability: Availability;
  contentAvailability?: Availability[];
  childSubtreeAvailability: Availability;
  /** Index into `propertyTables` of the per-tile metadata, if any. */
  tileMetadata?: number;
  propertyTables?: {
    class: string;
    count: number;
    properties: Record<string, { values: number }>;
  }[];
};

/** The tileset's metadata schema, as far as tile metadata needs it. */
export type MetadataSchema = {
  classes?: Record<
    string,
    {
      properties?: Record<
        string,
        { semantic?: string; componentType?: string }
      >;
    }
  >;
};

/** What a tile's own metadata says of it, where it says anything. */
type TileMetadata = { region?: Region; geometricError?: number };

type ImplicitTiling = {
  subdivisionScheme: string;
  subtreeLevels: number;
  availableLevels: number;
  subtrees: { uri: string };
};

type Region = [number, number, number, number, number, number];

type Tile = {
  boundingVolume: { region: Region };
  geometricError: number;
  refine?: string;
  content?: { uri: string };
  contents?: { uri: string }[];
  children?: Tile[];
};

const SUBTREE_MAGIC = "subt";
const HEADER_BYTES = 24;

/** A quadtree cell's index along a Morton curve: x in even bits, y in odd. */
export const morton = (x: number, y: number) => {
  let m = 0;
  for (let bit = 0; bit < 16; bit++) {
    m += ((x >> bit) & 1) * 2 ** (2 * bit);
    m += ((y >> bit) & 1) * 2 ** (2 * bit + 1);
  }
  return m;
};

/** Where `level` starts in a subtree's tile bitstream. */
const levelOffset = (level: number) => (4 ** level - 1) / 3;

type Bits = (index: number) => boolean;

/**
 * Reads a `.subtree` file into tests for each kind of availability, and a
 * lookup of each available tile's own bounding region and geometric error
 * where the subtree stores them as tile metadata.
 */
export const parseSubtree = (
  bytes: ArrayBuffer,
  loadBuffer: (uri: string) => Promise<ArrayBuffer>,
  schema?: MetadataSchema,
  subtreeLevels = 1,
) => {
  const view = new DataView(bytes);
  const magic = new TextDecoder().decode(bytes.slice(0, 4));
  if (magic !== SUBTREE_MAGIC) throw new Error("Not a subtree file");
  const jsonLength = Number(view.getBigUint64(8, true));
  const binaryLength = Number(view.getBigUint64(16, true));
  const json = JSON.parse(
    new TextDecoder().decode(
      bytes.slice(HEADER_BYTES, HEADER_BYTES + jsonLength),
    ),
  );
  const internal = bytes.slice(
    HEADER_BYTES + jsonLength,
    HEADER_BYTES + jsonLength + binaryLength,
  );

  return (async () => {
    const buffers: ArrayBuffer[] = await Promise.all(
      (json.buffers ?? []).map((b: { uri?: string }) =>
        b.uri ? loadBuffer(b.uri) : Promise.resolve(internal),
      ),
    );
    const bitsOf = (availability: Availability): Bits => {
      if (availability.bitstream === undefined) {
        const on = availability.constant === 1;
        return () => on;
      }
      const bufferView = json.bufferViews[availability.bitstream];
      const data = new Uint8Array(
        buffers[bufferView.buffer],
        bufferView.byteOffset ?? 0,
        bufferView.byteLength,
      );
      return (index) => ((data[index >> 3] >> (index & 7)) & 1) === 1;
    };
    const subtree: Subtree = json;
    const tile = bitsOf(subtree.tileAvailability);

    // Tile metadata has one row per available tile, in the order of the
    // tiles' indices; properties are found by their semantics.
    const table =
      subtree.tileMetadata === undefined
        ? undefined
        : subtree.propertyTables?.[subtree.tileMetadata];
    const classProperties = table
      ? (schema?.classes?.[table.class]?.properties ?? {})
      : {};
    const valuesOf = (semantic: string) => {
      const entry = Object.entries(classProperties).find(
        ([, property]) => property.semantic === semantic,
      );
      const column = entry && table?.properties[entry[0]];
      if (!entry || !column) return undefined;
      const bufferView = json.bufferViews[column.values];
      const start = buffers[bufferView.buffer].slice(
        bufferView.byteOffset ?? 0,
        (bufferView.byteOffset ?? 0) + bufferView.byteLength,
      );
      return entry[1].componentType === "FLOAT32"
        ? Array.from(new Float32Array(start))
        : Array.from(new Float64Array(start));
    };
    const regions = valuesOf("TILE_BOUNDING_REGION");
    const errors = valuesOf("TILE_GEOMETRIC_ERROR");
    const rows = new Map<number, number>();
    if (table) {
      const tilesInSubtree = levelOffset(subtreeLevels);
      for (let index = 0; index < tilesInSubtree; index++) {
        if (tile(index)) rows.set(index, rows.size);
      }
    }
    const metadataOf = (index: number): TileMetadata | undefined => {
      const row = rows.get(index);
      if (row === undefined) return undefined;
      const r = regions?.slice(row * 6, row * 6 + 6);
      return {
        region:
          r?.length === 6 ? [r[0], r[1], r[2], r[3], r[4], r[5]] : undefined,
        geometricError: errors?.[row],
      };
    };

    return {
      tile,
      contents: (subtree.contentAvailability ?? []).map(bitsOf),
      childSubtree: bitsOf(subtree.childSubtreeAvailability),
      metadataOf,
    };
  })();
};

const fillTemplate = (template: string, level: number, x: number, y: number) =>
  template
    .replaceAll("{level}", String(level))
    .replaceAll("{x}", String(x))
    .replaceAll("{y}", String(y));

/**
 * The tileset at `url` with implicit tiling written out as explicit tiles,
 * with absolute URLs so it can be served from anywhere. Tilesets without
 * implicit tiling come back unchanged.
 */
export const toExplicitTileset = async (
  url: string,
  fetchBytes: (url: string) => Promise<ArrayBuffer> = async (u) => {
    const res = await fetch(u);
    if (!res.ok) throw new Error(`${res.status} for ${u}`);
    return res.arrayBuffer();
  },
) => {
  const tileset = JSON.parse(new TextDecoder().decode(await fetchBytes(url)));
  const root = tileset.root;
  const implicit: ImplicitTiling | undefined = root?.implicitTiling;
  if (!implicit) return tileset;
  if (implicit.subdivisionScheme !== "QUADTREE") {
    throw new Error(`${implicit.subdivisionScheme} tiling is not supported`);
  }

  const absolute = (uri: string) => new URL(uri, url).toString();
  const templates: string[] = root.contents
    ? root.contents.map((c: { uri: string }) => c.uri)
    : root.content
      ? [root.content.uri]
      : [];
  const [west, south, east, north, minHeight, maxHeight] = root.boundingVolume
    .region as Region;
  const { subtreeLevels, availableLevels } = implicit;

  const regionOf = (level: number, x: number, y: number): Region => {
    const n = 2 ** level;
    const lonStep = (east - west) / n;
    const latStep = (north - south) / n;
    return [
      west + x * lonStep,
      south + y * latStep,
      west + (x + 1) * lonStep,
      south + (y + 1) * latStep,
      minHeight,
      maxHeight,
    ];
  };

  // Builds the tile at a global cell, reading availability from the subtree
  // whose root is (subtreeLevel, sx, sy).
  const buildSubtree = async (
    subtreeLevel: number,
    sx: number,
    sy: number,
  ): Promise<Tile | undefined> => {
    const subtreeUrl = absolute(
      fillTemplate(implicit.subtrees.uri, subtreeLevel, sx, sy),
    );
    const bytes = await fetchBytes(subtreeUrl);
    const subtree = await parseSubtree(
      bytes,
      (uri) => fetchBytes(new URL(uri, subtreeUrl).toString()),
      tileset.schema,
      subtreeLevels,
    );

    const build = async (
      local: number,
      lx: number,
      ly: number,
    ): Promise<Tile | undefined> => {
      const level = subtreeLevel + local;
      if (level >= availableLevels) return undefined;
      const index = levelOffset(local) + morton(lx, ly);
      if (!subtree.tile(index)) return undefined;

      const x = sx * 2 ** local + lx;
      const y = sy * 2 ** local + ly;
      // A tile's own metadata, where the writer stores it, overrides what
      // its level implies: a loose quadtree's tiles reach past their cells.
      const metadata = subtree.metadataOf(index);
      const tile: Tile = {
        boundingVolume: { region: metadata?.region ?? regionOf(level, x, y) },
        geometricError:
          metadata?.geometricError ?? root.geometricError / 2 ** level,
        refine: level === 0 ? root.refine : undefined,
      };
      const uris = templates
        .filter((_, n) => subtree.contents[n]?.(index))
        .map((t) => ({ uri: absolute(fillTemplate(t, level, x, y)) }));
      if (uris.length === 1) tile.content = uris[0];
      else if (uris.length > 1) tile.contents = uris;

      const children = await Promise.all(
        [0, 1].flatMap((dy) =>
          [0, 1].map((dx) => {
            const cx = lx * 2 + dx;
            const cy = ly * 2 + dy;
            if (local + 1 < subtreeLevels) return build(local + 1, cx, cy);
            // The child is the root of another subtree file.
            if (!subtree.childSubtree(morton(cx, cy))) return undefined;
            return buildSubtree(
              level + 1,
              sx * 2 ** (local + 1) + cx,
              sy * 2 ** (local + 1) + cy,
            );
          }),
        ),
      );
      const present = children.filter((c): c is Tile => !!c);
      if (present.length) tile.children = present;
      return tile;
    };

    return build(0, 0, 0);
  };

  const explicitRoot = await buildSubtree(0, 0, 0);
  const { implicitTiling: _, content: __, contents: ___, ...rest } = root;
  return {
    ...tileset,
    root: { ...rest, ...explicitRoot, refine: root.refine },
  };
};
