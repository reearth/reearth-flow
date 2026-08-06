/**
 * Holds the image bytes a feature carries, outside the React tree.
 *
 * A glTF read with embedded textures puts whole encoded images inside the
 * intermediate-data JSONL (`Raster::InMemory`), serialized as a JSON array of
 * integers — roughly 3.6 characters per source byte, repeated on every feature
 * that shares the texture. Left in the parsed feature they would be walked by
 * the table columnizer and the details-panel search, both of which
 * `JSON.stringify` every value they are handed.
 *
 * So the bytes are lifted out here at parse time and the feature keeps a
 * {@link RasterHandle} in their place. Identical images collapse onto one
 * entry, pixels live in a `Uint8Array` rather than a JS number array, and blob
 * URLs are minted only for images something actually displays.
 */

/** Marks the object that replaced a raster's payload in a parsed feature. */
export const RASTER_REF = "__rasterRef";

/** What a stripped `Raster::InMemory` payload leaves behind on the feature. */
export type RasterHandle = {
  [RASTER_REF]: string;
  mime_type: string;
  byteLength: number;
};

type RasterEntry = {
  mime: string;
  /** Null once the store's budget has been reached; the handle still resolves. */
  data: Uint8Array | null;
  byteLength: number;
  /** Data URLs of the files referencing this image, for eviction. */
  owners: Set<string>;
  objectUrl: string | null;
  objectUrlRefs: number;
};

/**
 * Ceiling on retained pixels. Past it, images are described but not kept, so a
 * pathological file degrades to "no thumbnail" instead of exhausting the tab.
 */
const MAX_RETAINED_BYTES = 128 * 1024 * 1024;

/** Bytes sampled when fingerprinting an image; see {@link fingerprint}. */
const FINGERPRINT_SAMPLES = 1024;

const FNV_OFFSET_BASIS = 0x811c9dc5;
const FNV_PRIME = 0x01000193;

const entries = new Map<string, RasterEntry>();
let retainedBytes = 0;

/**
 * Content fingerprint over a strided sample of the image. Paired with the
 * exact byte length in the key, so two images can only share an entry if they
 * are the same size *and* agree everywhere the sample looks.
 */
function fingerprint(data: Uint8Array): string {
  let hash = FNV_OFFSET_BASIS;
  const stride = Math.max(1, Math.ceil(data.length / FINGERPRINT_SAMPLES));
  for (let i = 0; i < data.length; i += stride) {
    hash ^= data[i];
    hash = Math.imul(hash, FNV_PRIME);
  }
  return (hash >>> 0).toString(36);
}

/**
 * Normalize whatever the engine wrote for the bytes. Today serde emits a JSON
 * array of integers; a base64 string is accepted too, so a switch to a compact
 * encoding needs no change here.
 */
function toBytes(raw: unknown): Uint8Array | null {
  if (raw instanceof Uint8Array) return raw;
  if (Array.isArray(raw)) return Uint8Array.from(raw as number[]);
  if (typeof raw === "string") {
    try {
      const binary = atob(raw);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      return bytes;
    } catch {
      return null;
    }
  }
  return null;
}

/**
 * Take an image out of a parsed feature and return the handle that replaces
 * it. `owner` is the data URL of the file being read, so the image can be
 * dropped when that file leaves the query cache.
 */
export function internRaster(
  owner: string,
  mimeType: unknown,
  raw: unknown,
): RasterHandle | null {
  const data = toBytes(raw);
  if (!data) return null;

  const mime =
    typeof mimeType === "string" ? mimeType : "application/octet-stream";
  const key = `${data.length.toString(36)}-${fingerprint(data)}`;

  const existing = entries.get(key);
  if (existing) {
    existing.owners.add(owner);
    return {
      [RASTER_REF]: key,
      mime_type: existing.mime,
      byteLength: existing.byteLength,
    };
  }

  const withinBudget = retainedBytes + data.length <= MAX_RETAINED_BYTES;
  if (withinBudget) retainedBytes += data.length;

  entries.set(key, {
    mime,
    data: withinBudget ? data : null,
    byteLength: data.length,
    owners: new Set([owner]),
    objectUrl: null,
    objectUrlRefs: 0,
  });

  return { [RASTER_REF]: key, mime_type: mime, byteLength: data.length };
}

/** Whether a value is a handle this store put on a feature. */
export function isRasterHandle(value: unknown): value is RasterHandle {
  return (
    typeof value === "object" &&
    value !== null &&
    typeof (value as RasterHandle)[RASTER_REF] === "string"
  );
}

export type RasterInfo = {
  mime: string;
  byteLength: number;
  /** False when the store's budget meant the pixels were not kept. */
  retained: boolean;
};

export function getRasterInfo(ref: string): RasterInfo | null {
  const entry = entries.get(ref);
  if (!entry) return null;
  return {
    mime: entry.mime,
    byteLength: entry.byteLength,
    retained: entry.data !== null,
  };
}

/**
 * Blob URL for a stored image, created on first use. Every call that returns a
 * URL must be paired with {@link releaseObjectUrl}, so a component that shows
 * a thumbnail releases it on unmount.
 */
export function acquireObjectUrl(ref: string): string | null {
  const entry = entries.get(ref);
  if (!entry?.data) return null;

  if (!entry.objectUrl) {
    entry.objectUrl = URL.createObjectURL(
      new Blob([entry.data as BlobPart], { type: entry.mime }),
    );
  }
  entry.objectUrlRefs += 1;
  return entry.objectUrl;
}

export function releaseObjectUrl(ref: string): void {
  const entry = entries.get(ref);
  if (!entry?.objectUrl) return;

  entry.objectUrlRefs -= 1;
  if (entry.objectUrlRefs <= 0) {
    URL.revokeObjectURL(entry.objectUrl);
    entry.objectUrl = null;
    entry.objectUrlRefs = 0;
  }
}

/**
 * Drop an entry outright, ignoring `objectUrlRefs`.
 *
 * Eviction deliberately wins over display: a file leaving the query cache takes
 * its images with it. A thumbnail still on screen from that file breaks, which
 * needs the details panel open on one of nine-plus cached files, and costs a
 * broken image rather than anything worse — `releaseObjectUrl` finds no entry
 * afterwards and returns, so there is no double revoke. The refcount is for
 * {@link releaseObjectUrl}, where several components may show one image.
 */
function discard(key: string, entry: RasterEntry): void {
  if (entry.objectUrl) URL.revokeObjectURL(entry.objectUrl);
  if (entry.data) retainedBytes -= entry.byteLength;
  entries.delete(key);
}

/**
 * Drop one file's claim on the images it contributed. An image shared with
 * another cached file survives; one only this file referenced is released.
 */
export function releaseOwner(owner: string): void {
  for (const [key, entry] of entries) {
    if (!entry.owners.delete(owner)) continue;
    if (entry.owners.size === 0) discard(key, entry);
  }
}

/** Drop everything. Used on teardown and between tests. */
export function clearRasterStore(): void {
  for (const [key, entry] of [...entries]) discard(key, entry);
  entries.clear();
  retainedBytes = 0;
}

/** Retained-pixel total, for diagnostics and tests. */
export function retainedRasterBytes(): number {
  return retainedBytes;
}
