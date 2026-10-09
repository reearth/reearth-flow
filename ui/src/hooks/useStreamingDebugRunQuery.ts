import type { QueryClient } from "@tanstack/react-query";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useMemo, useRef, useState } from "react";

import { describeGeometry, isNextFormat } from "@flow/lib/intermediateData";
import { streamDecompressZstdJsonl } from "@flow/utils/compression";
import { intermediateDataTransform } from "@flow/utils/jsonl/transformIntermediateData";
import { streamJsonl } from "@flow/utils/streaming";
import type { StreamingProgress } from "@flow/utils/streaming";

export type SupportedDataTypes = "geojson" | "jsonl";

// Simple check for compression files, but this will be removed if all files are compressed
function isCompressedUrl(url: string): boolean {
  const lowerUrl = url.toLowerCase();
  return lowerUrl.endsWith(".zst");
}

type GeometryType = string | null;

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
 * is 17 MB of CityGML LOD1 boxes and 867 MB of dense LOD2 solids. The second
 * case exhausts the browser tab before the table shows anything.
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
  // Legacy CityGML keeps its rings under `gmlGeometries` and has no
  // `coordinates`, so reading only the latter would leave it uncounted.
  const gmlGeometries =
    geometry.gmlGeometries ?? geometry.value?.cityGmlGeometry?.gmlGeometries;
  if (Array.isArray(gmlGeometries)) {
    const ring = (value: unknown) => (Array.isArray(value) ? value.length : 0);
    let total = 0;
    for (const geom of gmlGeometries) {
      for (const polygon of geom?.polygons ?? []) {
        total += ring(polygon?.exterior);
        for (const hole of polygon?.interior ?? []) total += ring(hole);
      }
    }
    return total;
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
    totalFeatures: number;
    isLoading: boolean;
    isComplete: boolean;
    progress: { bytesProcessed: number; featuresProcessed: number };
    hasMore: boolean;
    error: Error | null;
  }>({
    data: [],
    detectedGeometryType: null,
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

          const positions = featurePositions((transformed as any).geometry);
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

            // Detect the geometry type from the first batch
            if (!detectedGeometryType && result.data.length > 0) {
              const analysis = analyzeDataType(result.data);
              detectedGeometryType = analysis.geometryType;
            }

            // Only store data up to the display limits, but always update
            // progress.
            const shouldUpdateData = retainForDisplay(result.data);

            // Update streaming state
            setStreamingState((prev) => ({
              ...prev,
              data: shouldUpdateData ? [...streamData] : prev.data,
              detectedGeometryType,
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
            }

            // Only store data up to the display limits
            const shouldUpdateData = retainForDisplay(result.data);

            // Update streaming state
            setStreamingState((prev) => ({
              ...prev,
              data: shouldUpdateData ? [...streamData] : prev.data,
              detectedGeometryType,
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
