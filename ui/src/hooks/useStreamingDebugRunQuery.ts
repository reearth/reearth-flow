import { useQuery, useQueryClient, QueryClient } from "@tanstack/react-query";
import { useEffect, useMemo, useRef, useState } from "react";

import {
  describeGeometry,
  isNextFormat,
  type GeometryDescription,
} from "@flow/lib/intermediateData";
import { streamDecompressZstdJsonl } from "@flow/utils/compression";
import { intermediateDataTransform } from "@flow/utils/jsonl/transformIntermediateData";
import { hasGeoJsonForm } from "@flow/utils/jsonl/transformNextFeature";
import { streamJsonl } from "@flow/utils/streaming";
import type { StreamingProgress } from "@flow/utils/streaming";

export type SupportedDataTypes = "geojson" | "jsonl";

// Simple check for compression files, but this will be removed if all files are compressed
function isCompressedUrl(url: string): boolean {
  const lowerUrl = url.toLowerCase();
  return lowerUrl.endsWith(".zst");
}

type GeometryType = string | null;

type VisualizerType = "2d-map" | "3d-map" | "3d-model" | null;

type UseStreamingDebugRunQueryOptions = {
  enabled?: boolean;
  batchSize?: number;
  chunkSize?: number;
  displayLimit?: number;
  onProgress?: (progress: StreamingProgress) => void;
  onError?: (error: Error) => void;
};

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

type Drawable = {
  kind: "2d" | "3d";
  /** True when the coordinates are not on the earth; see {@link isModelSpace}. */
  modelSpace: boolean;
};

/**
 * Whether a frame places coordinates on the earth or in model space.
 *
 * The OBJ and glTF readers emit `CoordinateFrame::Euclidean` because those
 * formats carry no CRS — glTF's reader says so outright, "no CRS, so every leaf
 * uses `CoordinateFrame::Euclidean`". A tangent plane anchored in a Euclidean
 * base is the same. Neither is longitude and latitude, so neither belongs on a
 * globe.
 */
function isModelSpace(frame: unknown): boolean {
  if (frame === "Euclidean") return true;
  if (!frame || typeof frame !== "object") return false;

  const tangent = (frame as Record<string, unknown>).Tangent;
  if (tangent && typeof tangent === "object") {
    return isModelSpace((tangent as Record<string, unknown>).base);
  }
  return false;
}

/**
 * What a geometry draws as, or null when nothing in it draws.
 *
 * Descends into a collection's members: a CityGML feature is a collection of
 * per-LOD members, so judging it by its own kind would conclude the file has
 * nothing to draw.
 */
function drawable(described: GeometryDescription): Drawable | null {
  if (described.kind === "2d" || described.kind === "3d") {
    if (!hasGeoJsonForm(described.variant)) return null;
    const leaf = (described.value ?? {}) as Record<string, unknown>;
    return { kind: described.kind, modelSpace: isModelSpace(leaf.frame) };
  }

  if (described.kind === "collection") {
    const members = ((described.value as { members?: unknown[] } | undefined)
      ?.members ?? []) as unknown[];
    for (const member of members) {
      const found = drawable(describeGeometry(member));
      if (found) return found;
    }
  }

  return null;
}

/**
 * Predominant type and viewer for new-format data.
 *
 * The transform emits GeoJSON for every leaf that has one, in both embedding
 * dimensions, so the viewer choice is just which map. A point cloud or a CSG
 * tree yields only a summary, and a file of nothing but those gets no viewer
 * rather than an empty globe.
 */
function analyzeNextFormat(sample: any[]): {
  geometryType: GeometryType;
  visualizerType: VisualizerType;
} {
  const described = sample.map((feature) =>
    describeGeometry(feature?.geometry),
  );

  const labels = described
    .filter((entry) => entry.kind !== "none" && entry.kind !== "unknown")
    .map((entry) => entry.label || entry.variant || "Unknown");

  const drawables = described
    .map(drawable)
    .filter((entry): entry is Drawable => entry !== null);

  if (drawables.length === 0) {
    return { geometryType: singleType(labels), visualizerType: null };
  }

  // Model-space 3D goes to the model viewer, not a map: an OBJ or glTF read
  // has no CRS, so its coordinates would land at null island on a globe. The
  // legacy path reached the same place by sniffing an `OBJ`/`glTF` source
  // attribute; the frame states it outright. Left to the maps in 2D, which is
  // what the legacy path did there regardless of CRS.
  const models = drawables.filter(
    (entry) => entry.modelSpace && entry.kind === "3d",
  ).length;
  if (models * 2 >= drawables.length) {
    return { geometryType: singleType(labels), visualizerType: "3d-model" };
  }

  // 3D coordinates carry an altitude the 2D map drops, so a predominantly 3D
  // file gets the globe.
  const threeD = drawables.filter((entry) => entry.kind === "3d").length;
  return {
    geometryType: singleType(labels),
    visualizerType: threeD * 2 >= drawables.length ? "3d-map" : "2d-map",
  };
}

/**
 * The geometry label and viewer for a file, from a sample of its raw features.
 *
 * Exported for testing: it decides which viewer opens and is the one place
 * both geometry formats have to agree.
 */
export function analyzeDataType(features: any[]): {
  geometryType: GeometryType;
  visualizerType: VisualizerType;
} {
  if (features.length === 0)
    return { geometryType: null, visualizerType: null };

  // Check first few features to determine predominant type
  const sampleSize = Math.min(10, features.length);
  const sample = features.slice(0, sampleSize);

  if (sample.some(isNextFormat)) return analyzeNextFormat(sample);

  const typeCounts: Record<string, number> = {};
  let hasObjGltfSource = false;

  for (let i = 0; i < sampleSize; i++) {
    const feature = features[i];
    const type = detectGeometryType(feature);
    const source = feature?.attributes?.source;

    if (type && type !== "Unknown") {
      typeCounts[type] = (typeCounts[type] || 0) + 1;
    }

    // Check for OBJ/glTF source
    if (source === "OBJ" || source === "glTF") {
      hasObjGltfSource = true;
    }
  }

  // Return most common type, or null if no geometry types found
  const entries = Object.entries(typeCounts);
  if (entries.length === 0) return { geometryType: null, visualizerType: null };

  const predominantType = entries.reduce((a, b) =>
    typeCounts[a[0]] > typeCounts[b[0]] ? a : b,
  )[0] as GeometryType;

  // If we have mixed types or mostly unknown, return null instead of confusing info
  const totalGeometryFeatures = Object.values(typeCounts).reduce(
    (sum, count) => sum + count,
    0,
  );
  if (totalGeometryFeatures < sampleSize / 2) {
    return { geometryType: null, visualizerType: null }; // Less than half have recognizable geometry
  }

  // Determine visualizer based on geometry type + source
  let visualizerType: VisualizerType = null;

  if (predominantType === "FlowGeometry2D") {
    visualizerType = "2d-map";
  } else if (predominantType === "CityGmlGeometry") {
    visualizerType = "3d-map";
  } else if (predominantType === "FlowGeometry3D") {
    visualizerType = hasObjGltfSource ? "3d-model" : "3d-map";
  }

  // The viewer has to pick one, but the label does not: name the type only
  // when the sample agrees on it.
  const named = entries.map(([type]) => LEGACY_TYPE_LABELS[type] ?? type);

  return { geometryType: singleType(named), visualizerType };
}

// Smart cache management to prevent memory issues with multiple files
function manageCacheSize(queryClient: QueryClient) {
  const MAX_CACHED_FILES = 8; // Limit to 8 cached files max
  const cache = queryClient.getQueryCache();

  // Get all streaming queries (exclude metadata queries)
  const streamingQueries = cache
    .getAll()
    .filter(
      (query: any) =>
        query.queryKey[0] === "streamingDataUrl" &&
        !query.queryKey.includes("metadata"),
    );

  if (streamingQueries.length > MAX_CACHED_FILES) {
    // Sort by cache time (oldest first)
    const sortedQueries = streamingQueries
      .map((query: any) => ({
        query,
        cachedAt: query.state.data?.cachedAt || 0,
      }))
      .sort((a, b) => a.cachedAt - b.cachedAt);

    // Remove oldest cached files beyond the limit
    const queriesToRemove = sortedQueries.slice(
      0,
      streamingQueries.length - MAX_CACHED_FILES,
    );

    queriesToRemove.forEach(({ query }) => {
      console.log("Removing old streaming cache for:", query.queryKey[1]);
      queryClient.removeQueries({ queryKey: query.queryKey });
    });
  }
}

/**
 * Positions the panel holds before it stops taking features.
 *
 * `displayLimit` counts features, which is the wrong unit for geometry that
 * varies by two orders of magnitude between files. Measured against the
 * transform's output, a retained position costs ~220 bytes, so 2000 features
 * is 17 MB of CityGML LOD1 boxes and 867 MB of dense LOD2 solids — and Cesium
 * then allocates its own positions and geometry instances on top. The second
 * case exhausts the renderer process before anything draws.
 *
 * This bounds the same thing `displayLimit` bounds, in the unit that actually
 * costs. It is deliberately generous: at ~220 bytes a position this is ~220 MB,
 * which no ordinary file reaches. Whatever it leaves out is already reported —
 * the table's footer reads "Rows: shown / total".
 */
const DISPLAY_POSITION_LIMIT = 1_000_000;

/**
 * Positions a converted geometry holds.
 *
 * Descends to the ring and takes its length rather than counting positions one
 * by one, so this costs a property read per ring, not per coordinate.
 */
function positionsIn(coordinates: unknown): number {
  if (!Array.isArray(coordinates) || coordinates.length === 0) return 0;
  // A position: `[lon, lat]` or `[lon, lat, z]`.
  if (typeof coordinates[0] === "number") return 1;
  // A ring, or any other flat list of positions.
  if (typeof (coordinates[0] as never[])[0] === "number") {
    return coordinates.length;
  }
  let total = 0;
  for (const item of coordinates) total += positionsIn(item);
  return total;
}

/** Positions a transformed feature holds, across every geometry in it. */
export function featurePositions(geometry: any): number {
  if (!geometry) return 0;
  if (Array.isArray(geometry.geometries)) {
    return geometry.geometries.reduce(
      (total: number, member: any) => total + featurePositions(member),
      0,
    );
  }
  return positionsIn(geometry.coordinates);
}

export const useStreamingDebugRunQuery = (
  dataUrl: string,
  options: UseStreamingDebugRunQueryOptions = {},
): {
  fileContent: any;
  fileType: SupportedDataTypes;
  isLoading: boolean;
  [key: string]: any;
} => {
  const {
    enabled = true,
    batchSize = 1000,
    chunkSize = 64 * 1024,
    displayLimit = 2000,
    onProgress,
    onError,
  } = options;

  const queryClient = useQueryClient();
  const queryKey = useMemo(() => ["streamingDataUrl", dataUrl], [dataUrl]);
  const abortControllerRef = useRef<AbortController>(null);

  // State for progressive streaming updates
  const [streamingState, setStreamingState] = useState<{
    data: any[];
    detectedGeometryType: GeometryType;
    visualizerType: VisualizerType;
    totalFeatures: number;
    isLoading: boolean;
    isComplete: boolean;
    progress: { bytesProcessed: number; featuresProcessed: number };
    hasMore: boolean;
    error: Error | null;
  }>({
    data: [],
    detectedGeometryType: null,
    visualizerType: null,
    totalFeatures: 0,
    isLoading: false,
    isComplete: false,
    progress: { bytesProcessed: 0, featuresProcessed: 0 },
    hasMore: false,
    error: null,
  });

  // Main streaming query - handles caching and final storage
  const streamingQuery = useQuery({
    queryKey,
    queryFn: async () => {
      if (!dataUrl) return null;

      let detectedGeometryType: GeometryType = null;
      let detectedVisualizerType: VisualizerType = null;
      const streamData: any[] = [];
      let totalFeatures = 0;
      let displayedPositions = 0;

      /**
       * Transform a batch and keep what fits, by feature count and by the
       * geometry those features carry. Reports whether anything was kept.
       */
      const retainForDisplay = (batch: any[]): boolean => {
        if (
          streamData.length >= displayLimit ||
          displayedPositions >= DISPLAY_POSITION_LIMIT
        ) {
          return false;
        }

        const room = displayLimit - streamData.length;
        let added = false;

        for (const feature of batch.slice(0, room)) {
          let transformed;
          try {
            transformed = intermediateDataTransform(feature);
          } catch (error) {
            console.warn("Failed to transform feature:", error, feature);
            transformed = feature;
          }

          const positions =
            featurePositions((transformed as any).geometry) +
            featurePositions((transformed as any).lodDetail?.geometry);
          if (displayedPositions + positions > DISPLAY_POSITION_LIMIT) break;
          streamData.push(transformed);
          displayedPositions += positions;
          added = true;
          if (displayedPositions >= DISPLAY_POSITION_LIMIT) break;
        }

        return added;
      };
      let isComplete = false;
      let progress = { bytesProcessed: 0, featuresProcessed: 0 };

      // Create abort controller for this query
      const controller = new AbortController();
      abortControllerRef.current = controller;

      // Initialize streaming state
      setStreamingState((prev) => ({
        ...prev,
        isLoading: true,
        error: null,
      }));

      try {
        // Check if file is compressed
        if (isCompressedUrl(dataUrl)) {
          // COMPRESSED FILES (.jsonl.zst) - Stream decompression
          console.log("📦 Streaming compressed file:", dataUrl);

          const streamGenerator = streamDecompressZstdJsonl(dataUrl, {
            batchSize,
            signal: controller.signal,
            onProgress: (streamProgress) => {
              progress = {
                bytesProcessed: streamProgress.bytesDownloaded,
                featuresProcessed: streamProgress.featuresProcessed,
              };
              onProgress?.(progress);
            },
          });

          // Process stream with progressive updates
          for await (const result of streamGenerator) {
            totalFeatures = result.progress.featuresProcessed;

            // Detect geometry type and visualizer from first batch
            if (!detectedGeometryType && result.data.length > 0) {
              const analysis = analyzeDataType(result.data);
              detectedGeometryType = analysis.geometryType;
              detectedVisualizerType = analysis.visualizerType;
            }

            // Only store data up to the display limits, but always update
            // progress.
            const shouldUpdateData = retainForDisplay(result.data);

            // Update streaming state
            setStreamingState((prev) => ({
              ...prev,
              data: shouldUpdateData ? [...streamData] : prev.data,
              detectedGeometryType,
              visualizerType: detectedVisualizerType,
              totalFeatures,
              progress: result.progress,
              hasMore: totalFeatures > streamData.length,
              isComplete: result.isComplete,
              isLoading: !result.isComplete,
            }));

            if (result.isComplete) {
              isComplete = true;
              break;
            }
          }
        } else {
          // UNCOMPRESSED FILES (.jsonl) - Use existing streaming
          console.log("📊 Streaming uncompressed file:", dataUrl);

          const streamGenerator = await streamJsonl(dataUrl, {
            batchSize,
            chunkSize,
            signal: controller.signal,
            onProgress: (streamProgress) => {
              progress = streamProgress;
              onProgress?.(streamProgress);
            },
            onError,
          });

          // Process stream with progressive updates
          for await (const result of streamGenerator) {
            totalFeatures = result.progress.featuresProcessed;

            // Detect geometry type from first batch
            if (!detectedGeometryType && result.data.length > 0) {
              const analysis = analyzeDataType(result.data);
              detectedGeometryType = analysis.geometryType;
              detectedVisualizerType = analysis.visualizerType;
            }

            // Only store data up to the display limits
            const shouldUpdateData = retainForDisplay(result.data);

            // Update streaming state
            setStreamingState((prev) => ({
              ...prev,
              data: shouldUpdateData ? [...streamData] : prev.data,
              detectedGeometryType,
              visualizerType: detectedVisualizerType,
              totalFeatures,
              progress: result.progress,
              hasMore: totalFeatures > streamData.length,
              isComplete: result.isComplete,
              isLoading: !result.isComplete,
            }));

            if (result.isComplete) {
              isComplete = true;
              break;
            }
          }
        }

        // Store final result in React Query cache
        const finalResult = {
          data: streamData,
          fileContent: streamData,
          detectedGeometryType,
          visualizerType: detectedVisualizerType,
          totalFeatures,
          isComplete,
          progress,
          hasMore: totalFeatures > streamData.length,
          error: null,
          cachedAt: Date.now(),
        };

        // Smart cache management to prevent memory issues
        manageCacheSize(queryClient);

        return finalResult;
      } catch (error) {
        if (error instanceof Error && error.name === "AbortError") {
          setStreamingState((prev) => ({
            ...prev,
            isLoading: false,
          }));
          throw error;
        }
        const err = error as Error;
        setStreamingState((prev) => ({
          ...prev,
          error: err,
          isLoading: false,
        }));
        throw error;
      }
    },
    enabled: enabled && !!dataUrl,
    staleTime: 30 * 60 * 1000, // 30 minutes
    gcTime: 2 * 60 * 60 * 1000, // 2 hours
    retry: false,
  });

  // Initialize from cache on mount/URL change
  useEffect(() => {
    if (dataUrl) {
      const cachedData = queryClient.getQueryData(queryKey) as any;
      if (cachedData && cachedData.isComplete) {
        // Use cached data immediately
        setStreamingState({
          data: cachedData.data || cachedData.fileContent || [],
          detectedGeometryType: cachedData.detectedGeometryType,
          visualizerType: cachedData.visualizerType || null,
          totalFeatures: cachedData.totalFeatures || 0,
          isLoading: false,
          isComplete: true,
          progress: cachedData.progress || {
            bytesProcessed: 0,
            featuresProcessed: 0,
          },
          hasMore: cachedData.hasMore || false,
          error: null,
        });
      } else {
        // Reset to empty state
        setStreamingState({
          data: [],
          detectedGeometryType: null,
          visualizerType: null,
          totalFeatures: 0,
          isLoading: false,
          isComplete: false,
          progress: { bytesProcessed: 0, featuresProcessed: 0 },
          hasMore: false,
          error: null,
        });
      }
    }
  }, [dataUrl, queryKey, queryClient]);

  // Create a separate query for metadata/initial check
  const metadataQuery = useQuery({
    queryKey: [...queryKey, "metadata"],
    queryFn: async () => {
      if (!dataUrl) return null;

      const response = await fetch(dataUrl, { method: "HEAD" });
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return {
        contentLength: response.headers.get("content-length"),
        contentType: response.headers.get("content-type"),
      };
    },
    enabled: enabled && !!dataUrl,
    staleTime: 30 * 60 * 1000, // 30 minutes
    gcTime: 60 * 60 * 1000, // 1 hour
  });

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }
    };
  }, []);

  // Memoize fileContent to prevent infinite re-renders
  const fileContent = useMemo(
    () => ({
      type: "FeatureCollection" as const,
      features: streamingState.data,
    }),
    [streamingState.data],
  );

  return {
    // Progressive streaming data (immediately available)
    ...streamingState,

    // Compatibility with existing interface
    fileContent,
    fileType: "geojson" as SupportedDataTypes,
    isLoading: streamingState.isLoading || metadataQuery.isLoading,

    // React Query compatibility
    data: streamingQuery.data,
    isError: streamingQuery.isError || metadataQuery.isError,
    error: streamingState.error || streamingQuery.error || metadataQuery.error,
  };
};
