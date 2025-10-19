import { useQuery, useQueryClient, QueryClient } from "@tanstack/react-query";
import { useEffect, useMemo, useRef, useState } from "react";

import { intermediateDataTransform } from "@flow/utils/jsonl/transformIntermediateData";
import { streamJsonl } from "@flow/utils/streaming";
import type { StreamingProgress } from "@flow/utils/streaming";

export type SupportedDataTypes = "geojson" | "jsonl";

type GeometryType =
  | "FlowGeometry2D"
  | "FlowGeometry3D"
  | "CityGmlGeometry"
  | "Unknown"
  | null;

type VisualizerType = "2d-map" | "3d-map" | "3d-model" | null;

type UseStreamingDebugRunQueryOptions = {
  enabled?: boolean;
  batchSize?: number;
  chunkSize?: number;
  displayLimit?: number;
  onProgress?: (progress: StreamingProgress) => void;
  onError?: (error: Error) => void;
};

function detectGeometryType(feature: any): GeometryType {
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

function analyzeDataType(features: any[]): {
  geometryType: GeometryType;
  visualizerType: VisualizerType;
} {
  if (features.length === 0) return { geometryType: null, visualizerType: null };

  // Check first few features to determine predominant type
  const sampleSize = Math.min(10, features.length);
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

  return { geometryType: predominantType, visualizerType };
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
    isStreaming: boolean;
    isComplete: boolean;
    progress: { bytesProcessed: number; featuresProcessed: number };
    hasMore: boolean;
    error: Error | null;
  }>({
    data: [],
    detectedGeometryType: null,
    visualizerType: null,
    totalFeatures: 0,
    isStreaming: false,
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
      let isComplete = false;
      let progress = { bytesProcessed: 0, featuresProcessed: 0 };

      // Create abort controller for this query
      const controller = new AbortController();
      abortControllerRef.current = controller;

      // Initialize streaming state
      setStreamingState((prev) => ({
        ...prev,
        isStreaming: true,
        error: null,
      }));

      try {
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

          // Detect geometry type and visualizer from first batch
          if (!detectedGeometryType && result.data.length > 0) {
            const analysis = analyzeDataType(result.data);
            detectedGeometryType = analysis.geometryType;
            detectedVisualizerType = analysis.visualizerType;
          }

          // Only store data up to display limit, but always update progress
          let shouldUpdateData = false;
          if (streamData.length < displayLimit) {
            const remainingToAdd = displayLimit - streamData.length;
            const dataToAdd = result.data.slice(0, remainingToAdd);

            const transformedData = dataToAdd.map((feature) => {
              try {
                return intermediateDataTransform(feature);
              } catch (error) {
                console.warn(
                  "Failed to transform streaming feature:",
                  error,
                  feature,
                );
                return feature;
              }
            });
            streamData.push(...transformedData);
            shouldUpdateData = true;
          }

          // Always update streaming state to show current progress and total count
          setStreamingState((prev) => ({
            ...prev,
            data: shouldUpdateData ? [...streamData] : prev.data, // Only update data if we added new items
            detectedGeometryType,
            visualizerType: detectedVisualizerType,
            totalFeatures, // Always update total count
            progress: result.progress, // Always update progress
            hasMore: totalFeatures > displayLimit,
            isComplete: result.isComplete,
            isStreaming: !result.isComplete,
          }));

          if (result.isComplete) {
            isComplete = true;
            break;
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
          isStreaming: false,
          progress,
          hasMore: totalFeatures > displayLimit,
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
            isStreaming: false,
          }));
          throw error;
        }
        const err = error as Error;
        setStreamingState((prev) => ({
          ...prev,
          error: err,
          isStreaming: false,
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
          isStreaming: false,
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
          isStreaming: false,
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
    isLoading: streamingQuery.isLoading || metadataQuery.isLoading,

    // React Query compatibility
    data: streamingQuery.data,
    isError: streamingQuery.isError || metadataQuery.isError,
    error: streamingState.error || streamingQuery.error || metadataQuery.error,
  };
};
