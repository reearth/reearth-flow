import { describe, expect, test } from "vitest";

import { morton, toExplicitTileset } from "./explicitTileset";

const BASE = "https://example.com/view/";

/** A `.subtree` file with one bitstream per availability, in that order. */
const subtreeFile = (
  tile: number[],
  content: number[],
  childSubtree: number[] | 0,
) => {
  const streams = [tile, content, ...(childSubtree ? [childSubtree] : [])];
  const bytes = streams.map((bits) => {
    const out = new Uint8Array(Math.max(1, Math.ceil(bits.length / 8)));
    bits.forEach((on, i) => {
      if (on) out[i >> 3] |= 1 << (i & 7);
    });
    return out;
  });
  let offset = 0;
  const bufferViews = bytes.map((b) => {
    const view = { buffer: 0, byteOffset: offset, byteLength: b.length };
    offset += b.length;
    return view;
  });
  const json = new TextEncoder().encode(
    JSON.stringify({
      buffers: [{ byteLength: offset }],
      bufferViews,
      tileAvailability: { bitstream: 0 },
      contentAvailability: [{ bitstream: 1 }],
      childSubtreeAvailability: childSubtree
        ? { bitstream: 2 }
        : { constant: 0 },
    }),
  );
  const out = new Uint8Array(24 + json.length + offset);
  out.set(new TextEncoder().encode("subt"), 0);
  const header = new DataView(out.buffer);
  header.setUint32(4, 1, true);
  header.setBigUint64(8, BigInt(json.length), true);
  header.setBigUint64(16, BigInt(offset), true);
  out.set(json, 24);
  let at = 24 + json.length;
  for (const b of bytes) {
    out.set(b, at);
    at += b.length;
  }
  return out.buffer;
};

const tilesetFile = (subtreeLevels: number, availableLevels: number) =>
  new TextEncoder().encode(
    JSON.stringify({
      asset: { version: "1.1" },
      geometricError: 100,
      root: {
        boundingVolume: { region: [0, 0, 4, 4, 0, 10] },
        geometricError: 100,
        refine: "ADD",
        content: { uri: "content/{level}/{x}/{y}.glb" },
        implicitTiling: {
          subdivisionScheme: "QUADTREE",
          subtreeLevels,
          availableLevels,
          subtrees: { uri: "subtrees/{level}.{x}.{y}.subtree" },
        },
      },
    }),
  ).buffer;

const serve = (files: Record<string, ArrayBuffer>) => async (url: string) => {
  const file = files[url.replace(BASE, "")];
  if (!file) throw new Error(`404 ${url}`);
  return file;
};

describe("morton", () => {
  test("puts x in the even bits and y in the odd bits", () => {
    expect(morton(1, 0)).toBe(1);
    expect(morton(0, 1)).toBe(2);
    expect(morton(1, 1)).toBe(3);
    expect(morton(2, 0)).toBe(4);
    expect(morton(3, 2)).toBe(13);
  });
});

describe("toExplicitTileset", () => {
  test("lists the available tiles of one subtree, with absolute content URLs", async () => {
    // Level 0: the root. Level 1, in Morton order: (0,0) (1,0) (0,1) (1,1).
    // Only (1,0) exists below the root, and only it has content.
    const tileset = await toExplicitTileset(
      `${BASE}tileset.json`,
      serve({
        "tileset.json": tilesetFile(2, 2),
        "subtrees/0.0.0.subtree": subtreeFile(
          [1, 0, 1, 0, 0],
          [0, 0, 1, 0, 0],
          0,
        ),
      }),
    );

    expect(tileset.root.implicitTiling).toBeUndefined();
    expect(tileset.root.content).toBeUndefined();
    expect(tileset.root.refine).toBe("ADD");
    expect(tileset.root.children).toEqual([
      {
        boundingVolume: { region: [2, 0, 4, 2, 0, 10] },
        geometricError: 50,
        content: { uri: `${BASE}content/1/1/0.glb` },
      },
    ]);
  });

  test("follows child subtrees into further files", async () => {
    // One level per subtree, so level 1 lives in its own files. Of the root's
    // four children only (0,1) has a subtree.
    const tileset = await toExplicitTileset(
      `${BASE}tileset.json`,
      serve({
        "tileset.json": tilesetFile(1, 2),
        "subtrees/0.0.0.subtree": subtreeFile([1], [1], [0, 0, 1, 0]),
        "subtrees/1.0.1.subtree": subtreeFile([1], [1], 0),
      }),
    );

    expect(tileset.root.content).toEqual({
      uri: `${BASE}content/0/0/0.glb`,
    });
    expect(tileset.root.children).toEqual([
      {
        boundingVolume: { region: [0, 2, 2, 4, 0, 10] },
        geometricError: 50,
        content: { uri: `${BASE}content/1/0/1.glb` },
      },
    ]);
  });

  test("takes each tile's bounds and error from its metadata, where stored", async () => {
    // Two levels in one subtree: the root and its child (1,0). Each has a
    // metadata row, in tile order, wider than its cell as a loose quadtree's.
    const errors = new Float64Array([80, 30]);
    const regions = new Float64Array([
      ...[-1, -1, 5, 5, 0, 20],
      ...[1.5, -0.5, 4.5, 2.5, 3, 9],
    ]);
    const bitstreams = [
      [1, 0, 1, 0, 0],
      [0, 0, 1, 0, 0],
    ].map((bits) => {
      const out = new Uint8Array(8);
      bits.forEach((on, i) => {
        if (on) out[i >> 3] |= 1 << (i & 7);
      });
      return out;
    });
    const views = [
      ...bitstreams,
      new Uint8Array(errors.buffer),
      new Uint8Array(regions.buffer),
    ];
    let offset = 0;
    const bufferViews = views.map((v) => {
      const view = { buffer: 0, byteOffset: offset, byteLength: v.length };
      offset += v.length;
      return view;
    });
    const json = new TextEncoder().encode(
      JSON.stringify({
        buffers: [{ byteLength: offset }],
        bufferViews,
        tileAvailability: { bitstream: 0 },
        contentAvailability: [{ bitstream: 1 }],
        childSubtreeAvailability: { constant: 0 },
        propertyTables: [
          {
            class: "tile",
            count: 2,
            properties: {
              geometricError: { values: 2 },
              boundingRegion: { values: 3 },
            },
          },
        ],
        tileMetadata: 0,
      }),
    );
    const jsonLength = Math.ceil(json.length / 8) * 8;
    const file = new Uint8Array(24 + jsonLength + offset);
    file.set(new TextEncoder().encode("subt"), 0);
    const header = new DataView(file.buffer);
    header.setUint32(4, 1, true);
    header.setBigUint64(8, BigInt(jsonLength), true);
    header.setBigUint64(16, BigInt(offset), true);
    file.set(json, 24);
    file.fill(0x20, 24 + json.length, 24 + jsonLength);
    let at = 24 + jsonLength;
    for (const v of views) {
      file.set(v, at);
      at += v.length;
    }

    const tilesetJson = JSON.parse(new TextDecoder().decode(tilesetFile(2, 2)));
    tilesetJson.schema = {
      classes: {
        tile: {
          properties: {
            geometricError: {
              componentType: "FLOAT64",
              semantic: "TILE_GEOMETRIC_ERROR",
            },
            boundingRegion: {
              componentType: "FLOAT64",
              semantic: "TILE_BOUNDING_REGION",
            },
          },
        },
      },
    };

    const tileset = await toExplicitTileset(
      `${BASE}tileset.json`,
      serve({
        "tileset.json": new TextEncoder().encode(JSON.stringify(tilesetJson))
          .buffer,
        "subtrees/0.0.0.subtree": file.buffer,
      }),
    );

    expect(tileset.root.boundingVolume.region).toEqual([-1, -1, 5, 5, 0, 20]);
    expect(tileset.root.geometricError).toBe(80);
    expect(tileset.root.children).toEqual([
      {
        boundingVolume: { region: [1.5, -0.5, 4.5, 2.5, 3, 9] },
        geometricError: 30,
        content: { uri: `${BASE}content/1/1/0.glb` },
      },
    ]);
  });

  test("returns a tileset without implicit tiling unchanged", async () => {
    const explicit = { asset: { version: "1.1" }, root: { children: [] } };
    const tileset = await toExplicitTileset(
      `${BASE}tileset.json`,
      serve({
        "tileset.json": new TextEncoder().encode(JSON.stringify(explicit))
          .buffer,
      }),
    );
    expect(tileset).toEqual(explicit);
  });
});
