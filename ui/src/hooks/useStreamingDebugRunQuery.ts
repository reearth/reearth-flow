import { useQuery, useQueryClient, QueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";
import { intermediateDataTransform } from "@flow/utils/jsonl/transformIntermediateData";
import { streamJsonl } from "@flow/utils/streaming";
import type { StreamingState, StreamingProgress } from "@flow/utils/streaming";

type GeometryType =
  | "FlowGeometry2D"
  | "FlowGeometry3D"
  | "CityGmlGeometry"
  | "Unknown"
  | null;

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

function analyzeDataType(features: any[]): GeometryType {
  if (features.length === 0) return null;

  // Check first few features to determine predominant type
  const sampleSize = Math.min(10, features.length);
  const typeCounts: Record<string, number> = {};

  for (let i = 0; i < sampleSize; i++) {
    const type = detectGeometryType(features[i]);
    if (type && type !== "Unknown") {
      typeCounts[type] = (typeCounts[type] || 0) + 1;
    }
  }

  // Return most common type, or null if no geometry types found
  const entries = Object.entries(typeCounts);
  if (entries.length === 0) return null;

  const predominantType = entries.reduce((a, b) =>
    typeCounts[a[0]] > typeCounts[b[0]] ? a : b,
  )[0] as GeometryType;

  // If we have mixed types or mostly unknown, return null instead of confusing info
  const totalGeometryFeatures = Object.values(typeCounts).reduce(
    (sum, count) => sum + count,
    0,
  );
  if (totalGeometryFeatures < sampleSize / 2) {
    return null; // Less than half have recognizable geometry
  }

  return predominantType;
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
  const [detectedFileType, setDetectedFileType] = useState<GeometryType>(null);

  const [streamingState, setStreamingState] = useState<
    StreamingState<any> & {
      totalFeatures: number;
    }
  >({
    data: [],
    isStreaming: false,
    isComplete: false,
    progress: {
      bytesProcessed: 0,
      featuresProcessed: 0,
    },
    error: null,
    hasMore: false,
    totalFeatures: 0,
  });

  // Create a separate query for metadata/initial check
  const metadataQuery = useQuery({
    queryKey: [...queryKey, "metadata"],
    queryFn: async () => {
      if (!dataUrl) return null;

      // Just fetch headers to check content length and type
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

  const startStreaming = useCallback(async () => {
    if (!dataUrl || streamingState.isStreaming) return;

    // Create new abort controller
    abortControllerRef.current = new AbortController();

    setStreamingState((prev) => ({
      ...prev,
      isStreaming: true,
      error: null,
    }));

    try {
      const streamGenerator = await streamJsonl(dataUrl, {
        batchSize,
        chunkSize,
        signal: abortControllerRef.current.signal,
        onProgress: (progress) => {
          setStreamingState((prev) => ({
            ...prev,
            progress,
            totalFeatures: progress.featuresProcessed,
          }));
          onProgress?.(progress);
        },
        onError: (error) => {
          setStreamingState((prev) => ({
            ...prev,
            error,
            isStreaming: false,
          }));
          onError?.(error);
        },
      });

      // Process stream
      for await (const result of streamGenerator) {
        // Always count all features for total
        const currentTotal = result.progress.featuresProcessed;

        // Detect file type from first batch if not already detected
        if (!detectedFileType && result.data.length > 0) {
          const type = analyzeDataType(result.data);
          setDetectedFileType(type);
        }

        // Update state with current progress and total count
        setStreamingState((prev) => {
          const currentDataLength = prev.data.length;

          // Only add data to display if we haven't reached our display limit
          let newDataToAdd: any[] = [];

          if (currentDataLength < displayLimit) {
            const remainingToDisplay = displayLimit - currentDataLength;
            const rawDataToAdd = result.data.slice(0, remainingToDisplay);

            // Apply intermediateDataTransform to match non-streaming behavior
            newDataToAdd = rawDataToAdd.map((feature) => {
              try {
                return intermediateDataTransform(feature);
              } catch (error) {
                console.warn(
                  "Failed to transform streaming feature:",
                  error,
                  feature,
                );
                return feature; // Return raw feature as fallback
              }
            });
          }

          const newState = {
            ...prev,
            data:
              newDataToAdd.length > 0
                ? [...prev.data, ...newDataToAdd]
                : prev.data,
            progress: result.progress,
            totalFeatures: currentTotal,
            isComplete: result.isComplete,
            hasMore: result.hasMore,
            isStreaming: !result.isComplete,
          };

          // Update query cache immediately with the new state
          if (result.isComplete) {
            queryClient.setQueryData(queryKey, {
              data: newState.data,
              progress: result.progress,
              isComplete: result.isComplete,
              hasMore: result.hasMore,
              fileContent: newState.data,
              type:
                detectedFileType || analyzeDataType(newState.data) || "jsonl",
              totalFeatures: currentTotal,
              detectedGeometryType: detectedFileType,
              cachedAt: Date.now(),
            });

            // Smart cache invalidation - limit total cached files
            manageCacheSize(queryClient);
          }

          return newState;
        });

        // If we've hit our display limit, we continue streaming but only for counting
        // (no need to break - let it count the full file for totalFeatures)

        // If streaming is complete, break the loop
        if (result.isComplete) {
          break;
        }
      }
    } catch (error) {
      if (error instanceof Error && error.name === "AbortError") {
        // Stream was aborted (likely due to URL change), clear any partial data
        setStreamingState((prev) => ({
          ...prev,
          isStreaming: false,
          // Keep existing data if we were just switching files
        }));
        // Don't cache partial streaming results when aborted
      } else {
        const err = error as Error;
        setStreamingState((prev) => ({
          ...prev,
          error: err,
          isStreaming: false,
        }));
        onError?.(err);
      }
    }
  }, [
    dataUrl,
    batchSize,
    chunkSize,
    displayLimit,
    onProgress,
    onError,
    queryClient,
    queryKey,
    streamingState.isStreaming,
    detectedFileType,
  ]);

  const resetStreaming = useCallback(() => {
    // Stop any ongoing streaming
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
      abortControllerRef.current = null;
    }

    setStreamingState({
      data: [],
      isStreaming: false,
      isComplete: false,
      progress: {
        bytesProcessed: 0,
        featuresProcessed: 0,
      },
      error: null,
      hasMore: false,
      totalFeatures: 0,
    });
    setDetectedFileType(null);
    queryClient.removeQueries({ queryKey });
  }, [queryClient, queryKey]);

  // Auto-start streaming when enabled and URL changes (but not if we have cached data)
  useEffect(() => {
    if (
      enabled &&
      dataUrl &&
      !streamingState.isStreaming &&
      streamingState.data.length === 0
    ) {
      // Check if we have cached data before starting streaming
      const cachedData = queryClient.getQueryData(queryKey);
      if (!cachedData || !(cachedData as any).isComplete) {
        // Small delay to ensure any abort operations have completed
        const timeoutId = setTimeout(() => {
          startStreaming();
        }, 100);

        return () => clearTimeout(timeoutId);
      }
    }
  }, [
    enabled,
    dataUrl,
    startStreaming,
    streamingState.isStreaming,
    streamingState.data.length,
    queryClient,
    queryKey,
  ]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }
    };
  }, []);

  // Reset state and check for cached data when URL changes
  useEffect(() => {
    // First, stop any ongoing streaming when URL changes
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
      abortControllerRef.current = null;
    }

    if (dataUrl) {
      // Always reset state first when URL changes
      const defaultState = {
        data: [],
        isStreaming: false,
        isComplete: false,
        progress: {
          bytesProcessed: 0,
          featuresProcessed: 0,
        },
        error: null,
        hasMore: false,
        totalFeatures: 0,
      };

      // Reset detected file type
      setDetectedFileType(null);

      // Check for cached data and override default state if available
      const cachedData = queryClient.getQueryData(queryKey);
      if (cachedData && (cachedData as any).isComplete) {
        const cacheDataArray =
          (cachedData as any).fileContent || (cachedData as any).data || [];
        // Initialize state from cached data
        setStreamingState({
          data: cacheDataArray,
          isStreaming: false,
          isComplete: true,
          progress: (cachedData as any).progress || {
            bytesProcessed: 0,
            featuresProcessed: cacheDataArray.length,
          },
          error: null,
          hasMore: false,
          totalFeatures:
            (cachedData as any).totalFeatures || cacheDataArray.length,
        });

        // Also set detected file type from cache
        if ((cachedData as any).detectedGeometryType) {
          setDetectedFileType((cachedData as any).detectedGeometryType);
        }
      } else {
        // Set default empty state
        setStreamingState(defaultState);
      }
    }
  }, [dataUrl, queryKey, queryClient]);

  // Also provide the cached data from React Query for compatibility
  const cachedQuery = useQuery({
    queryKey,
    queryFn: () => {
      // This function shouldn't actually run since we're manually setting the data
      return Promise.resolve(null);
    },
    enabled: false,
    staleTime: 30 * 60 * 1000, // 30 minutes
    gcTime: 2 * 60 * 60 * 1000, // 2 hours
  });

  // Memoize fileContent to prevent infinite re-renders
  const fileContent = useMemo(
    () => ({
      type: "FeatureCollection" as const,
      features: streamingState.data,
    }),
    [streamingState.data],
  );

  return {
    // Streaming-specific data
    ...streamingState,
    resetStreaming,

    // Metadata
    metadata: metadataQuery.data,
    detectedGeometryType: detectedFileType,

    // Compatibility with existing useFetchAndReadData interface
    fileContent,
    fileType: "geojson", // All streaming data normalized to geojson format
    isLoading: streamingState.isStreaming || metadataQuery.isLoading,

    // Additional query states
    isFetching: streamingState.isStreaming,

    // React Query compatibility
    data: cachedQuery.data,
    isError: !!streamingState.error || metadataQuery.isError,
    error: streamingState.error || metadataQuery.error,
  };
};
